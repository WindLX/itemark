//! 现场查询索引：每次查询都从当前 Markdown 记录重建，不使用数据库或持久缓存。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::domain::{format_id, id_number};
use crate::error::{Result, WorkspaceError};
use crate::record::Record;

#[derive(Debug, Default)]
pub struct ItemIndex {
    records: Vec<Record>,
}

impl ItemIndex {
    /// 扫描 items 目录，按稳定 ID 排序。
    pub fn scan(items_dir: &Path) -> Result<Self> {
        let mut paths = Vec::new();
        if items_dir.is_dir() {
            collect_markdown(items_dir, &mut paths)?;
        }
        paths.sort();
        let mut records = Vec::with_capacity(paths.len());
        for path in paths {
            records.push(Record::read(&path)?);
        }
        records.sort_by(|left, right| left.id().unwrap_or("").cmp(right.id().unwrap_or("")));
        Ok(Self { records })
    }

    #[must_use]
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 按项目范围内唯一的稳定 ID 查找记录。
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&Record> {
        self.records
            .iter()
            .find(|record| record.id().is_ok_and(|candidate| candidate == id))
    }

    pub fn require(&self, id: &str) -> Result<&Record> {
        self.find(id)
            .ok_or_else(|| WorkspaceError::usage(format!("unknown worklog item `{id}`")))
    }

    /// 分配一个项目范围内未使用的最小 ID。
    pub fn next_id(&self) -> Result<String> {
        let mut max = 0u64;
        for record in &self.records {
            if let Some(number) = id_number(record.id()?) {
                max = max.max(number);
            }
        }
        Ok(format_id(max + 1))
    }

    /// 记录 ID 是否已被占用（跨整个项目，不只同一 group）。
    #[must_use]
    pub fn id_taken(&self, id: &str) -> bool {
        self.find(id).is_some()
    }

    #[must_use]
    pub fn by_group(&self, group: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.group() == group)
            .collect()
    }

    #[must_use]
    pub fn by_kind(&self, kind: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.kind().is_ok_and(|candidate| candidate == kind))
            .collect()
    }

    /// 以该 ID 为父项的记录。
    #[must_use]
    pub fn children_of(&self, id: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.parent().as_deref() == Some(id))
            .collect()
    }

    /// 文本搜索：ID、标题、group、kind 与正文。
    #[must_use]
    pub fn search(&self, needle: &str) -> Vec<&Record> {
        let needle = needle.to_lowercase();
        self.records
            .iter()
            .filter(|record| {
                let haystack = format!(
                    "{}\n{}\n{}\n{}\n{}",
                    record.id().unwrap_or(""),
                    record.title(),
                    record.group(),
                    record.kind().unwrap_or(""),
                    record.body.render()
                );
                haystack.to_lowercase().contains(&needle)
            })
            .collect()
    }

    /// 记录按 ID 建表，供引用完整性检查与视图使用。
    #[must_use]
    pub fn by_id(&self) -> BTreeMap<String, &Record> {
        let mut map = BTreeMap::new();
        for record in &self.records {
            if let Ok(id) = record.id() {
                map.insert(id.to_string(), record);
            }
        }
        map
    }
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|error| {
        WorkspaceError::runtime(format!("cannot read directory: {error}")).at(dir)
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| WorkspaceError::runtime(error.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, out)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            out.push(path);
        }
    }
    Ok(())
}
