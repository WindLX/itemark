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

/// 帮助文案固定在编译期，不随项目语言切换（`docs/cli.md` 的边界说明）。
#[test]
fn help_text_does_not_follow_the_project_language() {
    let project = Project::new();

    let translated = project.ok(&["--language", "en", "--help"]);
    assert!(translated.contains("用法："), "{translated}");
    assert!(translated.contains("打印帮助"), "{translated}");
    for english in ENGLISH_SCAFFOLDING {
        assert!(
            !translated.contains(english),
            "帮助不随 `--language` 切换，但仍出现 `{english}`：{translated}"
        );
    }
}
