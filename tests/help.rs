//! 帮助文案：clap 内置的 help/version 说明也要跟项目语言一致，不残留英文
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn built_in_help_text_is_localized() {
    let project = Project::new();

    for args in [
        vec!["--help"],
        vec!["add", "--help"],
        vec!["group", "--help"],
    ] {
        let help = project.ok(&args);
        assert!(
            !help.contains("Print help") && !help.contains("Print version"),
            "`{}` 不该残留英文内置说明：{help}",
            args.join(" ")
        );
        assert!(help.contains("打印帮助"), "`{}`：{help}", args.join(" "));
        assert!(help.contains("全局选项"), "`{}`：{help}", args.join(" "));
    }

    assert!(project.ok(&["--help"]).contains("打印版本"));
    assert!(project.ok(&["help", "add"]).contains("--kind <kind 名称>"));
}
