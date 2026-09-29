//! Clap 参数定义。
//!
//! 命令名、选项名与 JSON 键名保持稳定，不随项目语言变化；只有人读文本按项目语言
//! 呈现。

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "itemark",
    version,
    about = "本地 Itemark：以稳定 ID 管理工作事项、事实与术语"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

/// 所有子命令共享的取值来源：CLI 参数 → `itemark.toml` → 内置默认值。
#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// 项目目录；默认从当前目录向上查找 `itemark.toml`
    #[arg(long, global = true, help_heading = "全局选项", value_name = "目录")]
    pub project: Option<PathBuf>,

    /// 直接指定 Itemark root，覆盖项目配置
    #[arg(long, global = true, help_heading = "全局选项", value_name = "目录")]
    pub root: Option<PathBuf>,

    /// 人读输出的着色策略：auto（默认）仅在终端且未设置 NO_COLOR 时着色
    #[arg(
        long,
        global = true,
        help_heading = "全局选项",
        value_name = "auto|always|never",
        value_parser = parse_color
    )]
    pub color: Option<crate::style::Choice>,

    /// 以 JSON 呈现结果
    #[arg(long, global = true, help_heading = "全局选项")]
    pub json: bool,
}

/// 解析 `--color` 的取值。
fn parse_color(value: &str) -> Result<crate::style::Choice, String> {
    crate::style::Choice::parse(value)
        .ok_or_else(|| format!("expected one of auto, always, never, got `{value}`"))
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 初始化项目配置与推荐目录
    Init(InitArgs),
    /// 按指定 kind 添加记录；稳定 ID 由 Itemark 分配
    Add(AddArgs),
    /// 按稳定 ID 查看一条记录
    Show(ShowArgs),
    /// 列出记录，可按 group、kind 或状态过滤
    List(ListArgs),
    /// 搜索当前记录内容
    Search(SearchArgs),
    /// 定点更新字段或正文分节，也可移动 group
    Update(UpdateArgs),
    /// 追加一条带日期的进展
    Log(LogArgs),
    /// 废弃一条记录，保留 ID 与历史
    Drop(DropArgs),
    /// 恢复一条已废弃记录，保留原业务状态
    Restore(RestoreArgs),
    /// 一级 group 的查看与新建
    Group(GroupArgs),
    /// kind 定义的查看与检查
    Kind(KindArgs),
    /// 检查记录必填项、正文节、关系与完成依据
    Check(CheckArgs),
    /// 生成当前总览或交接摘要
    Summary(SummaryArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// 重写已存在的 `itemark.toml`，不删除任何记录
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// kind 名称；必须在项目配置中已声明
    #[arg(long, value_name = "kind 名称")]
    pub kind: String,

    /// 所属一级 group；必须在项目配置中已声明
    #[arg(long, value_name = "group 名称")]
    pub group: String,

    /// 记录标题
    #[arg(long, value_name = "文本")]
    pub title: Option<String>,

    /// 设置 kind 声明的字段：`--set 字段=值`，可重复
    #[arg(long = "set", value_name = "字段=值", value_parser = parse_assignment)]
    pub set: Vec<(String, String)>,

    /// 从文件读取正文，替代 kind 模板的正文骨架
    #[arg(long, value_name = "文件")]
    pub body: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ShowArgs {
    /// 稳定 ID
    #[arg(value_name = "ID")]
    pub id: String,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// 只看一个或多个 group；重复传入时多个值按 OR 匹配
    #[arg(long, value_name = "group 名称")]
    pub group: Vec<String>,

    /// 只看一个或多个 kind；重复传入时多个值按 OR 匹配
    #[arg(long, value_name = "kind 名称")]
    pub kind: Vec<String>,

    /// 只看一个或多个业务状态；重复传入时多个值按 OR 匹配，支持判定键或 kind 声明的原始值
    #[arg(long, value_name = "状态")]
    pub status: Vec<String>,

    /// 包含已废弃记录
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    /// 搜索文本，匹配 ID、标题、group、kind 与正文
    #[arg(value_name = "文本")]
    pub query: String,

    /// 包含已废弃记录
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// 稳定 ID
    #[arg(value_name = "ID")]
    pub id: String,

    /// 设置 kind 声明的字段：`--set 字段=值`，可重复
    #[arg(long = "set", value_name = "字段=值", value_parser = parse_assignment)]
    pub set: Vec<(String, String)>,

    /// 删除一个字段：`--unset 字段`，可重复
    #[arg(long = "unset", value_name = "字段")]
    pub unset: Vec<String>,

    /// 移动到另一个已声明的 group，ID 不变
    #[arg(long, value_name = "group 名称")]
    pub group: Option<String>,

    /// 替换一个正文分节：`--section 分节=内容`，可重复
    #[arg(long = "section", value_name = "分节=内容", value_parser = parse_assignment)]
    pub section: Vec<(String, String)>,

    /// 向分节追加一行：`--append 分节=行`，可重复
    #[arg(long = "append", value_name = "分节=行", value_parser = parse_assignment)]
    pub append: Vec<(String, String)>,

    /// 跳过写入前的内容比对，直接用本次内容覆盖（外部改动会被丢弃）
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct LogArgs {
    /// 稳定 ID
    #[arg(value_name = "ID")]
    pub id: String,

    /// 进展文本
    #[arg(value_name = "文本")]
    pub text: String,

    /// 记录日期，默认为今天（UTC）
    #[arg(long, value_name = "YYYY-MM-DD")]
    pub date: Option<String>,

    /// 写入的正文分节，默认「进展」
    #[arg(long, value_name = "分节")]
    pub section: Option<String>,
}

#[derive(Debug, Args)]
pub struct DropArgs {
    /// 稳定 ID
    #[arg(value_name = "ID")]
    pub id: String,

    /// 废弃原因，写入进展分节
    #[arg(long, value_name = "文本")]
    pub reason: Option<String>,
}

#[derive(Debug, Args)]
pub struct RestoreArgs {
    /// 稳定 ID
    #[arg(value_name = "ID")]
    pub id: String,
}

#[derive(Debug, Args)]
pub struct GroupArgs {
    #[command(subcommand)]
    pub command: GroupCommand,
}

#[derive(Debug, Subcommand)]
pub enum GroupCommand {
    /// 列出已声明的一级 group
    List,
    /// 新建一个一级 group
    Add(GroupAddArgs),
    /// 查看一个 group 的记录与 kind 分布
    Show(GroupShowArgs),
}

#[derive(Debug, Args)]
pub struct GroupAddArgs {
    /// group 名称
    #[arg(value_name = "名称")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct GroupShowArgs {
    /// group 名称
    #[arg(value_name = "名称")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct KindArgs {
    #[command(subcommand)]
    pub command: KindCommand,
}

#[derive(Debug, Subcommand)]
pub enum KindCommand {
    /// 列出已声明的 kind
    List,
    /// 查看一个 kind 的声明与模板
    Show(KindShowArgs),
    /// 检查 kind 声明与模板是否自洽
    Check(KindCheckArgs),
}

#[derive(Debug, Args)]
pub struct KindShowArgs {
    /// kind 名称；省略时列出全部声明
    #[arg(value_name = "kind 名称")]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct KindCheckArgs {
    /// 只检查一个 kind；省略时检查全部
    #[arg(value_name = "kind 名称")]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// 只检查一条记录
    #[arg(value_name = "ID")]
    pub id: Option<String>,
}

#[derive(Debug, Args)]
pub struct SummaryArgs {
    /// 生成时点标签，默认取当前时间（UTC）
    #[arg(long, value_name = "时间戳")]
    pub at: Option<String>,

    /// 显式保存快照到指定文件；默认只输出
    #[arg(long, value_name = "文件")]
    pub save: Option<PathBuf>,

    /// 以交接摘要形式呈现
    #[arg(long)]
    pub handoff: bool,
}

/// 解析 `字段=值`；缺失 `=` 时由 clap 报用法错误。
fn parse_assignment(text: &str) -> Result<(String, String), String> {
    let Some((key, value)) = text.split_once('=') else {
        return Err(format!("expected `key=value`, got `{text}`"));
    };
    if key.trim().is_empty() {
        return Err(format!("expected a non-empty key in `{text}`"));
    }
    Ok((key.trim().to_string(), value.to_string()))
}

/// 根据项目语言构建帮助命令树；命令名、选项名与参数值始终保持稳定。
#[must_use]
pub fn localized_command(language: &str) -> clap::Command {
    let english = crate::i18n::is_english(language);
    let mut command = <Cli as clap::CommandFactory>::command();
    command.build();
    localize(command, english, "")
}

fn localize(command: clap::Command, english: bool, parent: &str) -> clap::Command {
    let locale = if english { "en" } else { "zh-CN" };
    let current = command.get_name().to_string();
    let section_commands = if english { "Commands" } else { "命令" };
    let section_options = if english { "Options" } else { "选项" };
    let section_args = if english { "Arguments" } else { "参数" };
    let section_global = if english {
        "Global options"
    } else {
        "全局选项"
    };
    let template = if english {
        "{before-help}{about-with-newline}Usage: {usage}\n\n{all-args}{after-help}"
    } else {
        "{before-help}{about-with-newline}用法：{usage}\n\n{all-args}{after-help}"
    };
    let mut command = command
        .help_template(template)
        .subcommand_help_heading(section_commands)
        .mut_args(|arg| {
            let id = arg.get_id().as_str().to_string();
            let heading = if id == "help" || id == "version" {
                section_global
            } else if arg.is_positional() {
                section_args
            } else {
                section_options
            };
            let mut arg = arg.help_heading(heading);
            let help = if current == "list" && matches!(id.as_str(), "group" | "kind" | "status") {
                crate::i18n::optional_text(&format!("list_arg_{id}"), locale)
            } else {
                crate::i18n::argument_help(&id, locale)
            };
            if let Some(help) = help {
                arg = arg.help(help).long_help(help);
            }
            let value_name = if english {
                match id.as_str() {
                    "project" | "root" => Some("DIR"),
                    "body" | "save" => Some("FILE"),
                    "kind" | "group" | "name" => Some("NAME"),
                    "title" | "text" | "reason" | "query" => Some("TEXT"),
                    "unset" => Some("FIELD"),
                    "id" => Some("ID"),
                    "set" | "section" | "append" => Some("NAME=VALUE"),
                    "date" => Some("YYYY-MM-DD"),
                    "status" => Some("STATUS"),
                    "at" => Some("TIMESTAMP"),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(value_name) = value_name {
                arg = arg.value_name(value_name);
            }
            arg
        });
    let about_key = if parent == "group" || parent == "kind" {
        format!("{}_{}", parent, command.get_name())
    } else {
        format!("cmd_{}", command.get_name())
    };
    if let Some(about) = crate::i18n::optional_text(&about_key, locale) {
        command = command.about(about);
    }
    if command.get_name() == "itemark" {
        command = command.about(crate::i18n::text("app_about", locale));
    }
    if command
        .get_arguments()
        .any(|arg| arg.get_id().as_str() == "help")
    {
        let help = crate::i18n::text("help", locale);
        command = command.mut_arg("help", |arg| arg.help(help).long_help(help));
    }
    if command
        .get_arguments()
        .any(|arg| arg.get_id().as_str() == "version")
    {
        let version = crate::i18n::text("version", locale);
        command = command.mut_arg("version", |arg| arg.help(version).long_help(version));
    }
    command.mut_subcommands(|sub| localize(sub, english, &current))
}
