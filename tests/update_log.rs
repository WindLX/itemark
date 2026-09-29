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

    let path = project.item_file(&id);
    let mut original = fs::read_to_string(&path).expect("read record");
    original.push_str("\n## 补充说明\n\n这段内容必须保留。\n");
    fs::write(&path, original).expect("write record");

    project.ok(&["update", &id, "--section", "目标=新的目标内容"]);

    let updated = fs::read_to_string(project.item_file(&id)).expect("read updated record");
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

    let body = fs::read_to_string(project.item_file(&id)).expect("read record");
    assert!(body.contains("- 2024-01-02：第一步完成"), "{body}");
    assert!(body.contains("- 2024-03-04：第二步完成"), "{body}");
    assert!(
        body.find("第一步完成") < body.find("第二步完成"),
        "progress keeps chronological order"
    );
}

#[test]
fn title_update_renames_the_record_file_without_changing_its_id() {
    let project = Project::new();
    project.configure();
    let id = project.add_note("原始标题", "研究");
    let old_path = project.item_file(&id);
    assert!(
        old_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("原始标题")
    );

    project.ok(&["update", &id, "--set", "title=../新 标题:安全"]);
    let new_path = project.item_file(&id);
    assert_ne!(old_path, new_path);
    assert!(!old_path.exists());
    assert!(
        new_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("新-标题-安全")
    );
    assert!(project.ok(&["show", &id]).contains("新 标题:安全"));
}
