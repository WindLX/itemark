//! CLI 子命令编排。
//!
//! 读命令直接打开只读项目上下文；写命令在项目锁内完成一次写操作，写入前比对内容以
//! 发现外部改动。

pub mod args;
pub mod config_cmd;
pub mod group;
pub mod item;
pub mod kind_cmd;
pub mod summary;

use std::path::PathBuf;

use crate::error::{Result, WorkspaceError};
use crate::output::OutputMode;
use crate::workspace::Workspace;

use args::{Cli, Command, GroupCommand, KindCommand, ListArgs, SearchArgs, ShowArgs};

/// 一次调用的公共上下文：项目配置位置、root 覆盖与输出模式。
pub struct Context {
    pub project: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub language: Option<String>,
    pub mode: OutputMode,
}

impl Context {
    /// 读命令使用：只在需要时按配置构建现场视图。
    pub fn workspace(&self) -> Result<Workspace> {
        Workspace::open(self.config_path()?.as_path(), self.root.as_deref())
    }

    /// 写命令使用：进入项目锁，结束时释放。
    pub fn locked_workspace(&self) -> Result<Workspace> {
        Workspace::open_locked(self.config_path()?.as_path(), self.root.as_deref())
    }

    pub fn config_path(&self) -> Result<PathBuf> {
        config_cmd::find_config(self.project.as_deref())
    }
}

/// 执行一次 CLI 调用。
pub fn run(cli: Cli) -> Result<()> {
    let context = Context {
        project: cli.global.project.clone(),
        root: cli.global.root.clone(),
        language: cli.global.language.clone(),
        mode: OutputMode::from_json(cli.global.json),
    };
    dispatch(&context, cli.command)
}

fn dispatch(context: &Context, command: Command) -> Result<()> {
    match command {
        Command::Init(args) => config_cmd::init(context, &args),
        Command::Add(args) => item::add(context, &args),
        Command::Show(args) => show(context, &args),
        Command::List(args) => list(context, &args),
        Command::Search(args) => search(context, &args),
        Command::Update(args) => item::update(context, &args),
        Command::Log(args) => item::log(context, &args),
        Command::Drop(args) => item::drop(context, &args),
        Command::Restore(args) => item::restore(context, &args),
        Command::Group(nested) => match nested.command {
            GroupCommand::List => group::list(context),
            GroupCommand::Add(args) => group::add(context, &args),
            GroupCommand::Show(args) => group::show(context, &args),
        },
        Command::Kind(nested) => match nested.command {
            KindCommand::List => kind_cmd::list(context),
            KindCommand::Show(args) => kind_cmd::show(context, &args),
            KindCommand::Check(args) => kind_cmd::check(context, &args),
        },
        Command::Check(args) => check(context, &args),
        Command::Summary(args) => summary::summary(context, &args),
    }
}

fn show(context: &Context, args: &ShowArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let record = workspace.index().require(&args.id)?;
    item::print_record(context.mode, &language, workspace.config(), record)
}

fn list(context: &Context, args: &ListArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let mut records: Vec<&crate::record::Record> = workspace.index().records().iter().collect();
    if !args.all {
        records.retain(|record| !record.lifecycle().is_dropped());
    }
    if let Some(group) = args.group.as_deref() {
        if !workspace.config().has_group(group) {
            return Err(WorkspaceError::usage(format!(
                "unknown group `{group}`; declared groups: {}",
                declared_groups(workspace.config())
            )));
        }
        records.retain(|record| record.group() == group);
    }
    if let Some(kind) = args.kind.as_deref() {
        records.retain(|record| record.kind().is_ok_and(|name| name == kind));
    }
    if let Some(status) = args.status.as_deref() {
        // 既接受判定键（`done_unverified`、`none`），也接受 kind 声明的原始取值。
        let config = workspace.config();
        records.retain(|record| {
            crate::status::of(config, record).as_key() == status
                || record.status(config).as_deref() == Some(status)
        });
    }
    item::print_records(context.mode, &language, workspace.config(), &records)
}

fn search(context: &Context, args: &SearchArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let mut records = workspace.index().search(&args.query);
    if !args.all {
        records.retain(|record| !record.lifecycle().is_dropped());
    }
    item::print_records(context.mode, &language, workspace.config(), &records)
}

fn check(context: &Context, args: &args::CheckArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let report = match args.id.as_deref() {
        Some(id) => {
            let record = workspace.index().require(id)?;
            let known: Vec<&str> = workspace
                .index()
                .records()
                .iter()
                .filter_map(|record| record.id().ok())
                .collect();
            let mut report = crate::checks::CheckReport {
                issues: Vec::new(),
                checked: 1,
            };
            crate::checks::check_record(workspace.config(), record, id, &known, &mut report.issues);
            report
        }
        None => crate::checks::check_all(&workspace),
    };
    item::print_report(context.mode, &language, &report)
}

/// 已声明 group 的人读清单，供用法错误信息使用。
pub fn declared_groups(config: &crate::workspace::Config) -> String {
    if config.groups.is_empty() {
        "none declared".to_string()
    } else {
        config.groups.join(", ")
    }
}
