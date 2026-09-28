//! 查询：list / search 按 group、kind 与文本筛选
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn records_are_listed_and_searched_by_group_kind_and_text() {
    let project = Project::new();
    project.configure();
    let research = project.add_note("研究记录", "研究");
    let product = project.add_work("产品工作", "todo");

    let by_group = project.ok(&["list", "--group", "研究", "--json"]);
    assert!(by_group.contains(&research));
    assert!(!by_group.contains(&product));

    let by_kind = project.ok(&["list", "--kind", "work", "--json"]);
    assert!(by_kind.contains(&product));
    assert!(!by_kind.contains(&research));

    let searched = project.ok(&["search", "产品工作", "--json"]);
    assert!(searched.contains(&product));
    assert!(
        ids_in(&searched).len() == 1,
        "search returns only the matching record"
    );
}
