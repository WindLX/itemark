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
    project.add_work("着色", "in_progress");

    let plain = project.ok(&["show", "WL-0001", "--color", "never"]);
    assert!(
        plain.contains("标题："),
        "text output is unchanged: {plain}"
    );
    assert!(
        !plain.contains(ESC),
        "--color never must stay plain: {plain:?}"
    );

    let colored = project.ok(&["show", "WL-0001", "--color", "always"]);
    assert!(
        colored.contains(ESC),
        "--color always must emit ANSI escapes: {colored:?}"
    );
    assert!(
        colored.contains("WL-0001"),
        "styling keeps the payload readable: {colored:?}"
    );
}

#[test]
fn auto_stays_plain_when_stdout_is_not_a_terminal() {
    let project = Project::new();
    project.configure();
    project.add_work("默认", "todo");

    let output = project.ok(&["show", "WL-0001", "--color", "auto"]);
    assert!(
        !output.contains(ESC),
        "auto must not style piped output: {output:?}"
    );
}

#[test]
fn an_explicit_always_wins_over_no_color() {
    let project = Project::new();
    project.configure();
    project.add_work("覆盖", "todo");

    let output = project
        .command()
        .env("NO_COLOR", "1")
        .args(["show", "WL-0001", "--color", "always"])
        .output()
        .expect("run worklog CLI");
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
    project.add_work("JSON", "todo");

    let json = project.ok(&["show", "WL-0001", "--json", "--color", "always"]);
    assert!(
        !json.contains(ESC),
        "JSON must stay machine-readable: {json:?}"
    );
    serde_json::from_str::<serde_json::Value>(&json).expect("stdout is valid JSON");
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
