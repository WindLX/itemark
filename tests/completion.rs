//! 完成依据：完成值需要证据或显式未验证说明
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn a_done_record_needs_evidence_or_an_explicit_unverified_note() {
    let project = Project::new();
    project.configure();

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=缺证据",
        "--set",
        "status=done",
    ]);
    assert_eq!(
        code, 2,
        "a done record without evidence is refused: {output}"
    );

    let with_evidence = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=有证据",
        "--set",
        "status=done",
        "--set",
        "completion_note=已完成并通过检查",
        "--set",
        "completion_evidence=just ci 全绿",
        "--json",
    ]);
    let done = ids_in(&with_evidence);
    assert_eq!(done.len(), 1);

    let unverified = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=未验证",
        "--set",
        "status=done",
        "--set",
        "completion_note=未验证：尚未在真实项目跑过",
        "--json",
    ]);
    assert_eq!(ids_in(&unverified).len(), 1);

    let summary = project.ok(&["summary", "--json"]);
    assert!(summary.contains("\"done\""), "{summary}");
    assert!(
        summary.contains("done_unverified"),
        "unverified completions are reported separately: {summary}"
    );
}

#[test]
fn an_unverified_note_must_be_an_explicit_prefix() {
    let project = Project::new();
    project.configure();

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=偶然提到未验证",
        "--set",
        "status=done",
        "--set",
        "completion_note=这个改动没有未验证的假设",
    ]);
    assert_eq!(
        code, 2,
        "a note that merely mentions the word does not count: {output}"
    );

    let accepted = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=显式未验证",
        "--set",
        "status=done",
        "--set",
        "completion_note=unverified: 还没有在真实项目跑过",
        "--json",
    ]);
    assert_eq!(ids_in(&accepted).len(), 1);
}

#[test]
fn completion_is_checked_only_when_this_write_sets_it() {
    let project = Project::new();
    project.configure();
    project.write(
        "worklog/items/WL-0001.md",
        "---\nid: \"WL-0001\"\nkind: \"work\"\ngroup: \"研究\"\ntitle: \"手写记录\"\nstatus: \"done\"\n---\n\n## 目标\n\n手写。\n",
    );

    let (code, output) = project.fail(&["update", "WL-0001", "--set", "status=done"]);
    assert_eq!(
        code, 2,
        "writing the completion value still needs its grounds: {output}"
    );

    let updated = project.ok(&["update", "WL-0001", "--set", "title=改过的标题", "--json"]);
    assert!(
        updated.contains("改过的标题"),
        "an unrelated edit is not blocked by an old completion: {updated}"
    );
}
