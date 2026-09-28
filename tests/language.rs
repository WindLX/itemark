//! 项目语言：面向使用者的文案只从 output 层的语言 seam 出现
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

/// 文本里是否还有汉字：英文输出的固定文案不该出现中文句子（记录内容中的中文是数据）。
fn has_han(text: &str) -> bool {
    text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

#[test]
fn init_output_follows_the_project_language() {
    let zh = Project::new();
    let zh_output = zh.ok(&["init"]);
    assert!(zh_output.contains("已创建"), "{zh_output}");
    assert!(
        zh_output.contains("下一步：在 worklog.toml 中声明 kind 与 group"),
        "{zh_output}"
    );

    let en = Project::new();
    let en_output = en.ok(&["--language", "en", "init"]);
    assert!(en_output.contains("created"), "{en_output}");
    assert!(
        en_output.contains("Next: declare kinds and groups in worklog.toml"),
        "{en_output}"
    );
    assert!(
        !has_han(&en_output),
        "英文输出里没有中文固定文案：{en_output}"
    );
}

#[test]
fn check_output_follows_the_project_language() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("跨语言检查", "todo");
    project.ok(&["update", &id, "--section", "目标=目标里有一句说明"]);

    let en = project.ok(&["--language", "en", "check"]);
    assert!(en.contains("records checked"), "{en}");
    assert!(en.contains("check passed"), "{en}");
    assert!(!has_han(&en), "{en}");

    let en_kinds = project.ok(&["--language", "en", "kind", "check"]);
    assert!(en_kinds.contains("kinds checked"), "{en_kinds}");
    assert!(!has_han(&en_kinds), "{en_kinds}");
}

#[test]
fn record_text_uses_the_language_separator() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("跨语言记录", "todo");

    let zh = project.ok(&["show", &id]);
    assert!(zh.contains("标题：跨语言记录"), "{zh}");

    let en = project.ok(&["--language", "en", "show", &id]);
    assert!(en.contains("title: 跨语言记录"), "{en}");
    assert!(!en.contains('：'), "英文输出用半角冒号：{en}");
}
