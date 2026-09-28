//! Clap 参数定义。
//!
//! 命令名、选项名与 JSON 键名保持稳定，不随项目语言变化；只有人读文本按项目语言
//! 呈现。

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "worklog",
    version,
    about = "本地 Worklog：以稳定 ID 管理工作事项、事实与术语"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

/// 所有子命令共享的取值来源：CLI 参数 → `worklog.toml` → 内置默认值。
#[derive(Debug, Args)]
pub struct GlobalArgs {
    /// 项目目录；默认从当前目录向上查找 `worklog.toml`
    #[arg(long, global = true, help_heading = "全局选项", value_name = "目录")]
    pub project: Option<PathBuf>,

    /// 直接指定 Worklog root，覆盖项目配置
    #[arg(long, global = true, help_heading = "全局选项", value_name = "目录")]
    pub root: Option<PathBuf>,

    /// 显式指定项目语言，覆盖项目配置
    #[arg(long, global = true, help_heading = "全局选项", value_name = "BCP47")]
    pub language: Option<String>,

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
    /// 按指定 kind 添加记录；稳定 ID 由 Worklog 分配
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
    /// 重写已存在的 `worklog.toml`，不删除任何记录
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
    /// 只看某个 group
    #[arg(long, value_name = "group 名称")]
    pub group: Option<String>,

    /// 只看某个 kind
    #[arg(long, value_name = "kind 名称")]
    pub kind: Option<String>,

    /// 只看某个业务状态：判定键（todo/in_progress/blocked/done_unverified/done/none）或 kind 声明的原始取值
    #[arg(long, value_name = "状态")]
    pub status: Option<String>,

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

/// 帮助模板：与 clap 默认模板一致，只把硬编码的 `Usage:` 换成「用法：」。
const HELP_TEMPLATE: &str = "\
{before-help}{about-with-newline}
用法：{usage}

{all-args}{after-help}";

/// 构建帮助文案已本地化的命令树。
///
/// 命令名、选项名与取值保持稳定，不随语言变化；帮助文案固定在编译期，也不随
/// `--language` 切换。clap 自带的 `Usage:`、`Options:`、`Arguments:`、`Commands:` 都是
/// 硬编码英文，这里换成中文标题：段落标题由每个参数自己的 `help_heading` 决定，
/// `build()` 之后补上的内置参数也在改写范围内。
///
/// 这些内置参数由 clap 在构建命令树时补上，所以先 `build()` 再改写；`build()` 会递归
/// 构建并复制整棵命令树（clap 的慢路径），只在启动时跑一次。
#[must_use]
pub fn localized_command() -> clap::Command {
    let mut command = <Cli as clap::CommandFactory>::command();
    command.build();
    localize(command)
}

fn localize(command: clap::Command) -> clap::Command {
    let mut command = command
        .subcommand_help_heading("命令")
        .help_template(HELP_TEMPLATE)
        .mut_args(|arg| {
            if arg.get_help_heading().is_some() {
                arg
            } else if arg.get_id().as_str() == "help" || arg.get_id().as_str() == "version" {
                arg.help_heading("全局选项")
            } else if arg.is_positional() {
                arg.help_heading("参数")
            } else {
                arg.help_heading("选项")
            }
        });
    if command
        .get_arguments()
        .any(|arg| arg.get_id().as_str() == "help")
    {
        // 一并改写 `long_help`：任一参数带上长帮助后，clap 会给 `--help` 重新显示英文说明。
        command = command.mut_arg("help", |arg| arg.help("打印帮助").long_help("打印帮助"));
    }
    if command
        .get_arguments()
        .any(|arg| arg.get_id().as_str() == "version")
    {
        command = command.mut_arg("version", |arg| arg.help("打印版本").long_help("打印版本"));
    }
    if command
        .get_subcommands()
        .any(|sub| sub.get_name() == "help")
    {
        command = command.mut_subcommand("help", |sub| sub.about("打印帮助"));
    }
    command.mut_subcommands(localize)
}
