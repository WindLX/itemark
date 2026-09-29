//! 帮助文案：clap 内置的 help/version 说明与段落标题也要跟项目语言一致，不残留英文
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

/// clap 自带的脚手架文案（`Usage:`/`Options:`/`Arguments:`/`Commands:`/`[default: …]`）
/// 都是硬编码英文，必须被本地化覆盖。
const ENGLISH_SCAFFOLDING: [&str; 6] = [
    "Print help",
    "Print version",
    "Usage:",
    "Options:",
    "Arguments:",
    "Commands:",
];

#[test]
fn built_in_help_text_is_localized() {
    let project = Project::new();
    project.configure();

    for args in [
        vec!["--help"],
        vec!["add", "--help"],
        vec!["log", "--help"],
        vec!["group", "--help"],
        vec!["group", "add", "--help"],
        vec!["show", "--help"],
        vec!["update", "--help"],
    ] {
        let help = project.ok(&args);
        let invoked = args.join(" ");
        for english in ENGLISH_SCAFFOLDING {
            assert!(
                !help.contains(english),
                "`{invoked}` 不该残留英文脚手架 `{english}`：{help}"
            );
        }
        assert!(
            !help.contains("[default:"),
            "`{invoked}` 不该残留英文默认值标记：{help}"
        );
        assert!(help.contains("用法："), "`{invoked}`：{help}");
        assert!(help.contains("打印帮助"), "`{invoked}`：{help}");
    }

    let root = project.ok(&["--help"]);
    assert!(root.contains("打印版本"), "{root}");
    assert!(root.contains("命令"), "根帮助要有命令段落：{root}");
    assert!(root.contains("全局选项"), "{root}");
    assert!(project.ok(&["help", "add"]).contains("--kind <kind 名称>"));
}

#[test]
fn help_text_follows_project_language() {
    let project = Project::new();
    project.write("itemark.toml", &CONFIG.replace("zh-CN", "en"));
    let help = project.ok(&["--help"]);
    for chinese in ["用法：", "全局选项", "打印帮助", "打印版本", "初始化项目"]
    {
        assert!(
            !help.contains(chinese),
            "help should be English; found {chinese}: {help}"
        );
    }
    for english in [
        "Usage:",
        "Commands",
        "Options",
        "Global options",
        "Print help",
        "Initialize project",
        "Show this help message",
    ] {
        assert!(
            help.contains(english),
            "help should contain {english}: {help}"
        );
    }
    assert!(
        !help.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
        "{help}"
    );
    let subcommand = project.ok(&["group", "add", "--help"]);
    assert!(
        subcommand.contains("Create a top-level group"),
        "{subcommand}"
    );
    assert!(!subcommand.contains("一级 group"), "{subcommand}");
    assert!(
        !subcommand
            .chars()
            .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
        "{subcommand}"
    );
    for args in [
        vec!["add", "--help"],
        vec!["update", "--help"],
        vec!["kind", "check", "--help"],
        vec!["help", "add"],
    ] {
        let help = project.ok(&args);
        assert!(
            !help.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
            "{}: {help}",
            args.join(" ")
        );
    }
}
