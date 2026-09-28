//! 已加载的项目上下文与写事务。

pub mod config;
pub mod lock;

use std::path::{Path, PathBuf};

use crate::error::{Result, WorkspaceError};
use crate::record::Record;
use crate::record::index::ItemIndex;

pub use config::{CONFIG_FILE, Config, KindConfig};

use lock::ProjectLock;

pub struct Workspace {
    pub config: Config,
    index: ItemIndex,
    lock: Option<ProjectLock>,
}

impl Workspace {
    /// 读取项目配置，并从当前记录现场构建查询索引。默认只读，不加锁。
    pub fn open(config_path: &Path, root_override: Option<&Path>) -> Result<Self> {
        let config = Config::load(config_path, root_override)?;
        let index = ItemIndex::scan(&config.items_dir())?;
        Ok(Self {
            config,
            index,
            lock: None,
        })
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// 写命令入口：先确保 items 目录存在并取得项目锁，调用结束后由 Drop 释放。
    ///
    /// 与 `transaction` 的区别是锁的存活期覆盖整次命令调用，便于同一命令内多次读取
    /// 刚写入的记录。
    pub fn open_locked(config_path: &Path, root_override: Option<&Path>) -> Result<Self> {
        let config = Config::load(config_path, root_override)?;
        std::fs::create_dir_all(config.items_dir())
            .map_err(|error| WorkspaceError::from(error).at(&config.items_dir()))?;
        let lock = ProjectLock::acquire(&config.root)?;
        let index = ItemIndex::scan(&config.items_dir())?;
        Ok(Self {
            config,
            index,
            lock: Some(lock),
        })
    }

    #[must_use]
    pub fn index(&self) -> &ItemIndex {
        &self.index
    }

    /// 重新构建索引；写操作后使用，保证同一次调用内的读取看到最新记录。
    pub fn refresh(&mut self) -> Result<()> {
        self.index = ItemIndex::scan(&self.config.items_dir())?;
        Ok(())
    }

    /// 写入一条记录后就地更新内存索引。
    ///
    /// 传入的是刚写入磁盘的全文，因此不必再从磁盘读回；一次写操作只需在开始时扫一次项目。
    pub fn note_written(&mut self, path: &Path, text: &str) -> Result<()> {
        self.index.upsert(Record::parse(path, text.to_string())?);
        Ok(())
    }

    #[must_use]
    pub fn project_language(&self, cli: Option<&str>) -> String {
        self.config.effective_language(cli)
    }

    /// 在同一项目锁内完成一次写入，结束后重建索引。
    ///
    /// 锁定期间不接受其他 CLI 进程写入；外部编辑器不参与该锁，冲突由写入前的
    /// 内容比对发现。
    pub fn transaction<T>(
        &mut self,
        action: impl FnOnce(&mut Transaction<'_>) -> Result<T>,
    ) -> Result<T> {
        if self.lock.is_none() {
            std::fs::create_dir_all(self.config.items_dir())
                .map_err(|error| WorkspaceError::from(error).at(&self.config.items_dir()))?;
            self.lock = Some(ProjectLock::acquire(&self.config.root)?);
            self.refresh()?;
        }
        let result = {
            let mut transaction = Transaction { workspace: self };
            action(&mut transaction)
        };
        self.lock = None;
        result
    }
}

/// 一次写操作期间可用的记录读写入口。
pub struct Transaction<'w> {
    workspace: &'w mut Workspace,
}

impl Transaction<'_> {
    #[must_use]
    pub fn index(&self) -> &ItemIndex {
        self.workspace.index()
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        self.workspace.config()
    }

    /// 读取一条记录的当前状态；读取失败说明记录在锁外被外部进程改动或删除。
    pub fn current(&self, id: &str) -> Result<Record> {
        let path = self.item_path(id)?;
        Record::read(&path)
    }

    /// 记录文件的规范路径：`<root>/items/<ID>.md`。
    pub fn item_path(&self, id: &str) -> Result<PathBuf> {
        let record = self
            .workspace
            .index()
            .find(id)
            .ok_or_else(|| WorkspaceError::usage(format!("unknown worklog item `{id}`")))?;
        Ok(record.path.clone())
    }

    /// 已加载的当前索引视图，供记录间的引用检查使用。
    #[must_use]
    pub fn records(&self) -> &[Record] {
        self.workspace.index().records()
    }

    /// 分配项目范围内唯一的稳定 ID。
    pub fn allocate_id(&self) -> Result<String> {
        self.workspace.index().next_id()
    }

    /// 创建一条记录。ID 已存在时明确失败，不覆盖既有记录。
    pub fn create(&mut self, id: &str, text: &str) -> Result<PathBuf> {
        if self.workspace.index().id_taken(id) {
            return Err(WorkspaceError::usage(format!(
                "worklog item `{id}` already exists"
            )));
        }
        let path = self.workspace.config.items_dir().join(format!("{id}.md"));
        if path.exists() {
            return Err(
                WorkspaceError::runtime(format!("file already exists for `{id}`")).at(&path),
            );
        }
        write_new(&path, text)?;
        self.workspace.note_written(&path, text)?;
        Ok(path)
    }

    /// 覆盖一条既有记录；`expected` 是本次操作开始时读到的内容，用于发现外部改动。
    pub fn update(&mut self, id: &str, expected: Option<&str>, text: &str) -> Result<PathBuf> {
        let path = self.item_path(id)?;
        if let Some(expected) = expected {
            let current = std::fs::read_to_string(&path)
                .map_err(|error| crate::error::read_error(&path, error))?;
            if current != expected {
                return Err(WorkspaceError::runtime(format!(
                    "worklog item `{id}` changed outside this operation; \
                     re-read it and retry (use --force to overwrite anyway)"
                ))
                .at(&path));
            }
        }
        write_new(&path, text)?;
        self.workspace.note_written(&path, text)?;
        Ok(path)
    }
}

fn write_new(path: &Path, text: &str) -> Result<()> {
    let temp = path.with_extension("md.tmp");
    std::fs::write(&temp, text).map_err(|error| WorkspaceError::from(error).at(&temp))?;
    std::fs::rename(&temp, path).map_err(|error| WorkspaceError::from(error).at(path))?;
    Ok(())
}
