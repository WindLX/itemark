//! `--color`：人读输出的着色策略
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。测试进程的标准输出是管道，因此默认
//! `auto` 在这里就等同于「非终端」。

mod common;

use common::*;

/// ANSI 转义序列的开头。
const ESC: &str = "\u{1b}[";

#[test]
fn color_always_styles_text_and_never_keeps_it_plain() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("着色", "in_progress");

    let plain = project.ok(&["show", &id, "--color", "never"]);
    assert!(
        plain.contains("标题："),
        "text output is unchanged: {plain}"
    );
    assert!(
        !plain.contains(ESC),
        "--color never must stay plain: {plain:?}"
    );

    let colored = project.ok(&["show", &id, "--color", "always"]);
    assert!(
        colored.contains(ESC),
        "--color always must emit ANSI escapes: {colored:?}"
    );
    assert!(
        colored.contains(&id),
        "styling keeps the payload readable: {colored:?}"
    );
}

#[test]
fn auto_stays_plain_when_stdout_is_not_a_terminal() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("默认", "todo");

    let output = project.ok(&["show", &id, "--color", "auto"]);
    assert!(
        !output.contains(ESC),
        "auto must not style piped output: {output:?}"
    );
}

#[test]
fn an_explicit_always_wins_over_no_color() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("覆盖", "todo");

    let output = project
        .command()
        .env("NO_COLOR", "1")
        .args(["show", &id, "--color", "always"])
        .output()
        .expect("run Itemark CLI");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(
        text.contains(ESC),
        "an explicit --color always is an opt-in: {text:?}"
    );
}

#[test]
fn json_output_is_never_styled() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("JSON", "todo");

    let json = project.ok(&["show", &id, "--json", "--color", "always"]);
    assert!(
        !json.contains(ESC),
        "JSON must stay machine-readable: {json:?}"
    );
    serde_json::from_str::<serde_json::Value>(&json).expect("stdout is valid JSON");
}

#[test]
fn list_styles_hierarchy_but_keeps_json_clean() {
    let project = Project::new();
    project.configure();
    project.add_work("列表层级", "in_progress");

    let text = project.ok(&["list", "--color", "always"]);
    assert!(
        text.contains(ESC),
        "human list output should style headings and titles"
    );
    assert!(
        text.contains("列表层级"),
        "styling preserves the title: {text}"
    );

    let json = project.ok(&["list", "--json", "--color", "always"]);
    assert!(!json.contains(ESC), "JSON remains plain: {json:?}");
    serde_json::from_str::<serde_json::Value>(&json).expect("stdout is valid JSON");

    let no_color = project
        .command()
        .env("NO_COLOR", "1")
        .args(["list", "--color", "auto"])
        .output()
        .expect("run list with NO_COLOR");
    assert!(no_color.status.success());
    let text = String::from_utf8(no_color.stdout).expect("stdout is UTF-8");
    assert!(!text.contains(ESC), "NO_COLOR keeps the list plain: {text:?}");
}

#[test]
fn summary_and_check_follow_the_same_flag() {
    let project = Project::new();
    project.configure();
    project.add_work("总览", "in_progress");

    let summary = project.ok(&["summary", "--color", "always"]);
    assert!(
        summary.contains(ESC),
        "summary styles its human output: {summary:?}"
    );

    // 模板留下的空必填分节让 `check` 必然失败，所以看标准输出而不是退出码。
    let output = project.run(&["check", "--color", "always"]);
    let checked = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(
        checked.contains(ESC),
        "check styles its human output: {checked:?}"
    );
    assert!(
        !output.status.success(),
        "the empty required section is still reported"
    );
}
