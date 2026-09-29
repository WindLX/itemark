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

#[test]
fn an_empty_selection_says_so_instead_of_printing_nothing() {
    let project = Project::new();
    project.configure();
    project.add_note("研究记录", "研究");

    let zh = project.ok(&["list", "--group", "产品"]);
    assert_eq!(zh.trim(), "无匹配记录");

    // `search` 与 `list` 共用同一条空结果提示。
    let searched = project.ok(&["search", "查不到的词"]);
    assert_eq!(searched.trim(), "无匹配记录");

    project.write("itemark.toml", &CONFIG.replace("zh-CN", "en"));
    let en = project.ok(&["list", "--group", "产品"]);
    assert_eq!(en.trim(), "no matching records");
}

#[test]
fn long_titles_keep_list_and_summary_metadata_on_separate_lines() {
    let project = Project::new();
    project.configure();
    let title = "这是一个特意写得比较长的标题，用于确认列表中元数据不会被挤到标题后面";
    let id = project.add_work(title, "in_progress");

    let listed = project.ok(&["list"]);
    let mut lines = listed.lines();
    assert_eq!(lines.next(), Some(format!("{id} {title}").as_str()));
    let metadata = lines.next().expect("metadata line");
    assert!(
        metadata.starts_with("  kind：work · group：产品"),
        "{metadata}"
    );
    assert!(metadata.contains("状态：进行中"), "{metadata}");

    let summary = project.ok(&["summary"]);
    let summary_lines: Vec<_> = summary.lines().collect();
    assert!(summary_lines[1].starts_with("待办："));
    assert!(summary_lines[2].starts_with("进行中："), "{summary}");
    assert!(
        summary.contains(&format!("- {id} {title}\n  kind：work\n  group：产品")),
        "{summary}"
    );
}
