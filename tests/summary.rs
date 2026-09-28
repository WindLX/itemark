//! summary：默认只读，显式 --save 才写快照
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn summary_is_read_only_until_a_snapshot_is_requested() {
    let project = Project::new();
    project.configure();
    project.add_note("总览来源", "研究");

    project.ok(&["summary"]);
    assert!(
        !project.exists("worklog/summaries"),
        "the overview is not saved unless asked"
    );

    project.ok(&["summary", "--save", "worklog/summaries/handoff.md"]);
    let snapshot = project.read("worklog/summaries/handoff.md");
    assert!(snapshot.contains("generated_at"), "{snapshot}");
    assert!(snapshot.contains("WL-0001"), "{snapshot}");
}

#[test]
fn records_without_a_business_status_are_counted_separately() {
    let project = Project::new();
    project.configure();
    project.add_note("一条笔记", "研究");

    let json = project.ok(&["summary", "--json"]);
    assert!(json.contains("\"none\": 1"), "{json}");
    assert!(json.contains("\"todo\": 0"), "{json}");
}

#[test]
fn the_overview_labels_the_done_count_as_done() {
    let project = Project::new();
    project.configure();
    project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=已完成的事项",
        "--set",
        "status=done",
        "--set",
        "completion_note=已完成并通过检查",
        "--set",
        "completion_evidence=just ci 全绿",
    ]);

    let text = project.ok(&["summary"]);
    assert!(text.contains("已完成：1"), "{text}");
}

#[test]
fn handoff_reports_next_steps_and_differs_from_the_overview() {
    let project = Project::new();
    project.configure();
    let work = project.add_work("推进中的事项", "in_progress");
    project.ok(&["update", &work, "--section", "下一步=补集成测试"]);
    project.add_note("不进入交接的笔记", "研究");

    let overview = project.ok(&["summary"]);
    let handoff = project.ok(&["summary", "--handoff"]);
    assert!(handoff.contains("交接摘要"), "{handoff}");
    assert!(handoff.contains(&work), "{handoff}");
    assert!(handoff.contains("补集成测试"), "{handoff}");
    assert!(
        !handoff.contains("不进入交接的笔记"),
        "only actionable records enter the handoff: {handoff}"
    );
    assert_ne!(handoff, overview, "--handoff is not a copy of the overview");

    let json = project.ok(&["summary", "--handoff", "--json"]);
    assert!(json.contains("\"next_steps\""), "{json}");
    assert!(!json.contains("\"counts\""), "{json}");
}

#[test]
fn the_handoff_leaves_out_records_that_are_not_in_flight() {
    let project = Project::new();
    project.configure();
    let active = project.add_work("推进中的事项", "in_progress");
    project.ok(&["update", &active, "--section", "下一步=补集成测试"]);
    let todo = project.add_work("还没开始的事项", "todo");
    project.ok(&["update", &todo, "--section", "下一步=以后再排期"]);
    let added = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "产品",
        "--set",
        "title=已经完成的事项",
        "--set",
        "status=done",
        "--set",
        "completion_note=已完成并通过检查",
        "--set",
        "completion_evidence=just ci 全绿",
        "--json",
    ]);
    let done = ids_in(&added).first().cloned().expect("add returns an ID");
    project.ok(&["update", &done, "--section", "下一步=已经无需推进"]);

    let handoff = project.ok(&["summary", "--handoff"]);
    assert!(handoff.contains("补集成测试"), "{handoff}");
    assert!(handoff.contains("下一步（1）"), "{handoff}");
    assert!(
        !handoff.contains("以后再排期") && !handoff.contains("已经无需推进"),
        "todo and done records are not in flight: {handoff}"
    );

    let json = project.ok(&["summary", "--handoff", "--json"]);
    assert!(
        !json.contains("以后再排期") && !json.contains("已经无需推进"),
        "the handoff JSON follows the same rule: {json}"
    );
}
