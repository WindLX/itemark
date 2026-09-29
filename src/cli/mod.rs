//! CLI 子命令编排。
//!
//! 读命令直接打开只读项目上下文；写命令在项目锁内完成一次写操作，写入前比对内容以
//! 发现外部改动。

pub mod archive;
pub mod args;
pub mod config_cmd;
pub mod group;
pub mod item;
pub mod kind_cmd;
pub mod merge;
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
        mode: OutputMode::from_json(cli.global.json),
    };
    // 着色只在人读文本下有意义；JSON 必须保持机器可读。
    crate::style::enable(
        !context.mode.is_json()
            && cli.global.color.unwrap_or_default().wants_color(
                crate::style::stdout_is_terminal(),
                crate::style::no_color_requested(),
            ),
    );
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
        Command::Archive(args) => archive::archive(context, &args),
        Command::Unarchive(args) => archive::unarchive(context, &args),
        Command::Merge(args) => merge::merge(context, &args),
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
    let language = workspace.project_language();
    let record = workspace.index().require(&args.id)?;
    item::print_record(context.mode, &language, workspace.config(), record)
}

fn list(context: &Context, args: &ListArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let mut records: Vec<&crate::record::Record> = workspace.index().records().iter().collect();
    if !args.all {
        records.retain(|record| !record.lifecycle().is_dropped());
    }
    if args.archived {
        records.retain(|record| workspace.is_archived(record));
    } else if !args.include_archived {
        records.retain(|record| !workspace.is_archived(record));
    }
    let groups = unique_values(&args.group);
    for group in &groups {
        if !workspace.config().has_group(group) {
            return Err(WorkspaceError::usage(format!(
                "unknown group `{group}`; declared groups: {}",
                declared_groups(workspace.config())
            )));
        }
    }
    if !groups.is_empty() {
        records.retain(|record| groups.contains(&record.group()));
    }
    let kinds = unique_values(&args.kind);
    if !kinds.is_empty() {
        records.retain(|record| record.kind().is_ok_and(|name| kinds.contains(&name)));
    }
    let statuses = unique_values(&args.status);
    if !statuses.is_empty() {
        // 既接受判定键（`done_unverified`、`none`），也接受 kind 声明的原始取值。
        let config = workspace.config();
        records.retain(|record| {
            let state = crate::status::of(config, record).as_key();
            let raw_status = record.status(config);
            statuses
                .iter()
                .any(|status| state == *status || raw_status.as_deref() == Some(*status))
        });
    }
    let merge_roles = unique_values(&args.merge_role);
    if !merge_roles.is_empty() {
        records.retain(|record| {
            let is_source = !record.references("merged_into").is_empty();
            let is_result = !record.references("merged_from").is_empty();
            merge_roles.iter().any(|role| match *role {
                "source" => is_source,
                "result" => is_result,
                "none" => !is_source && !is_result,
                _ => false,
            })
        });
    }
    let reference_health = unique_values(&args.reference_health);
    if !reference_health.is_empty() {
        let known = workspace.index().by_id();
        let config = workspace.config();
        records.retain(|record| {
            reference_health
                .iter()
                .any(|expected| reference_health_for(config, record, &known) == *expected)
        });
    }
    if args.needs_review {
        records.retain(|record| {
            record
                .get("needs_review")
                .is_some_and(|value| value.display() == "true")
        });
    }
    let labels = crate::output::labels(&language);
    let mut filters = Vec::new();
    let or = crate::i18n::text("filter_or", &language);
    let and = crate::i18n::text("filter_and", &language);
    if !groups.is_empty() {
        filters.push(filter_description(labels.group_filter(), &groups, or));
    }
    if !kinds.is_empty() {
        filters.push(filter_description(labels.kind_filter(), &kinds, or));
    }
    if !statuses.is_empty() {
        filters.push(filter_description(labels.status_filter(), &statuses, or));
    }
    if !merge_roles.is_empty() {
        filters.push(filter_description(
            labels.merge_role_filter(),
            &merge_roles,
            or,
        ));
    }
    if !reference_health.is_empty() {
        filters.push(filter_description(
            labels.reference_health_filter(),
            &reference_health,
            or,
        ));
    }
    if args.needs_review {
        filters.push(labels.needs_review_filter().to_string());
    }
    if args.all {
        filters.push(labels.all_filter().to_string());
    }
    if args.archived {
        filters.push(labels.archived_only().to_string());
    } else if args.include_archived {
        filters.push(labels.include_archived().to_string());
    }
    let mut list_header = crate::output::fill(labels.list_count(), &[&records.len().to_string()]);
    if !filters.is_empty() {
        list_header.push('\n');
        list_header.push_str(&crate::output::fill(
            labels.list_filter(),
            &[&filters.join(&format!(" {and} "))],
        ));
    }
    item::print_records(
        context.mode,
        &language,
        workspace.config(),
        &records,
        Some(&list_header),
    )
}

fn reference_health_for(
    config: &crate::workspace::config::Config,
    record: &crate::record::Record,
    known_ids: &std::collections::BTreeMap<String, &crate::record::Record>,
) -> &'static str {
    let Ok(id) = record.id() else {
        return "error";
    };
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    crate::checks::check_record(config, record, id, known_ids, &mut issues, &mut warnings);
    if issues
        .iter()
        .any(|issue| issue.kind == crate::checks::IssueKind::BrokenReference)
    {
        "error"
    } else if warnings
        .iter()
        .any(|warning| warning.kind == crate::checks::IssueKind::DeprecatedReference)
    {
        "warning"
    } else {
        "ok"
    }
}

fn unique_values(values: &[String]) -> Vec<&str> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value.as_str()) {
            unique.push(value.as_str());
        }
    }
    unique
}

fn filter_description(template: &str, values: &[&str], or: &str) -> String {
    let separator = format!(" {or} ");
    crate::output::fill(template, &[&values.join(&separator)])
}

fn search(context: &Context, args: &SearchArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let mut records = workspace.index().search(&args.query);
    if !args.all {
        records.retain(|record| !record.lifecycle().is_dropped());
    }
    if !args.include_archived {
        records.retain(|record| !workspace.is_archived(record));
    }
    item::print_records(context.mode, &language, workspace.config(), &records, None)
}

fn check(context: &Context, args: &args::CheckArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let report = match args.id.as_deref() {
        Some(id) => {
            let record = workspace.index().require(id)?;
            let known = workspace.index().by_id();
            let mut report = crate::checks::CheckReport {
                issues: Vec::new(),
                warnings: Vec::new(),
                checked: 1,
            };
            crate::checks::check_record(
                workspace.config(),
                record,
                id,
                &known,
                &mut report.issues,
                &mut report.warnings,
            );
            report
        }
        None => crate::checks::check_all(&workspace),
    };
    item::print_report(context.mode, &language, item::ReportScope::Records, &report)
}

/// 已声明 group 的人读清单，供用法错误信息使用。
pub fn declared_groups(config: &crate::workspace::Config) -> String {
    if config.groups.is_empty() {
        "none declared".to_string()
    } else {
        config.groups.join(", ")
    }
}
