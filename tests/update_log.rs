//! 定点更新与进展追加：只改指定分节、保留既有历史
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn update_changes_only_the_requested_section() {
    let project = Project::new();
    project.configure();
    let id = project.add_note("定点更新", "研究");

    let path = "worklog/items/WL-0001.md";
    let mut original = project.read(path);
    original.push_str("\n## 补充说明\n\n这段内容必须保留。\n");
    project.write(path, &original);

    project.ok(&["update", &id, "--section", "目标=新的目标内容"]);

    let updated = project.read(path);
    assert!(
        updated.contains("新的目标内容"),
        "the section was rewritten"
    );
    assert!(
        updated.contains("这段内容必须保留。"),
        "untouched Markdown is preserved: {updated}"
    );
    assert!(updated.contains("定点更新"), "front matter is preserved");
    assert!(updated.contains(&id), "the stable ID is preserved");
}

#[test]
fn log_appends_dated_progress_without_losing_history() {
    let project = Project::new();
    project.configure();
    let id = project.add_note("进展记录", "研究");

    project.ok(&["log", &id, "第一步完成", "--date", "2024-01-02"]);
    project.ok(&["log", &id, "第二步完成", "--date", "2024-03-04"]);

    let body = project.read("worklog/items/WL-0001.md");
    assert!(body.contains("- 2024-01-02：第一步完成"), "{body}");
    assert!(body.contains("- 2024-03-04：第二步完成"), "{body}");
    assert!(
        body.find("第一步完成") < body.find("第二步完成"),
        "progress keeps chronological order"
    );
}
