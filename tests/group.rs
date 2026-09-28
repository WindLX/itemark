//! group：新增分组的声明与去重；跨组移动保留 ID 与引用
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn group_add_declares_a_new_group_and_rejects_duplicates() {
    let project = Project::new();
    project.configure();

    project.ok(&["group", "add", "设计"]);
    let listed = project.ok(&["group", "list", "--json"]);
    assert!(listed.contains("设计"));

    let (code, _) = project.fail(&["group", "add", "设计"]);
    assert_eq!(code, 2, "a duplicate group name is a usage error");

    project.ok(&[
        "add",
        "--kind",
        "project-note",
        "--group",
        "设计",
        "--set",
        "title=设计记录",
        "--set",
        "phase=doing",
    ]);
    let shown = project.ok(&["group", "show", "设计", "--json"]);
    assert!(shown.contains("design") || shown.contains("设计记录"));
}

#[test]
fn moving_a_record_to_another_group_keeps_its_id_and_references() {
    let project = Project::new();
    project.configure();
    let parent = project.add_note("父记录", "研究");
    let child = project.add_note("子记录", "研究");
    project.ok(&["update", &parent, "--section", "目标=父的目标"]);
    project.ok(&["update", &child, "--section", "目标=子的目标"]);
    project.ok(&["update", &child, "--set", &format!("parent={parent}")]);

    project.ok(&["update", &child, "--group", "产品"]);

    let shown = project.ok(&["show", &child, "--json"]);
    assert!(shown.contains(&child), "the ID survives the move");
    assert!(shown.contains("产品"));
    assert!(
        shown.contains(&parent),
        "references still point at the original ID"
    );
    project.ok(&["check"]);
}
