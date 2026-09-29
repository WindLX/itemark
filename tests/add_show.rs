//! add / show：写入一条记录并读回它的稳定 ID 与字段
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn add_then_show_a_record_from_a_user_defined_kind() {
    let project = Project::new();
    project.configure();

    let added = project.ok(&[
        "add",
        "--kind",
        "project-note",
        "--group",
        "研究",
        "--set",
        "title=跨会话记录",
        "--set",
        "phase=doing",
        "--json",
    ]);
    let id = ids_in(&added).first().expect("first project ID").clone();
    assert_eq!(id, "IM-1", "new IDs use the Itemark prefix without padding");

    let shown = project.ok(&["show", &id, "--json"]);
    assert!(shown.contains("跨会话记录"));
    assert!(shown.contains("project-note"));
    assert!(shown.contains("研究"));
}

#[test]
fn show_prints_each_field_once_and_labels_the_header() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("只打印一次", "todo");

    let shown = project.ok(&["show", &id]);
    for label in ["ID：", "kind：", "group：", "标题：", "状态："] {
        assert!(shown.contains(label), "the header is localised: {shown}");
    }
    assert!(
        !shown.contains("completion_note："),
        "blank optional fields stay hidden: {shown}"
    );
    assert!(
        !shown.contains("completion_evidence："),
        "blank optional fields stay hidden: {shown}"
    );
    for raw in ["title:", "status:"] {
        assert_eq!(
            shown.matches(raw).count(),
            0,
            "`{raw}` is already covered by a localised label: {shown}"
        );
    }
}
