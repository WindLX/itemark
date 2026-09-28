//! 生命周期：drop / restore 保留 ID、历史与业务状态
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn drop_and_restore_keep_the_id_and_the_business_status() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("待废弃", "in_progress");

    project.ok(&["drop", &id, "--reason", "不再推进"]);
    let dropped = project.ok(&["show", &id, "--json"]);
    assert!(dropped.contains("dropped"), "{dropped}");

    let listed = project.ok(&["list", "--json"]);
    assert!(
        !listed.contains(&id),
        "dropped records stay out of the list"
    );
    let all = project.ok(&["list", "--all", "--json"]);
    assert!(all.contains(&id));

    project.ok(&["restore", &id]);
    let restored = project.ok(&["show", &id, "--json"]);
    assert!(restored.contains("in_progress"), "the status is preserved");
    assert!(restored.contains(&id), "the ID is preserved");
    assert!(
        project
            .read("worklog/items/WL-0001.md")
            .contains("不再推进"),
        "the drop reason stays in the history"
    );
}
