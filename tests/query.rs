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
    assert!(zh.contains("共 0 条记录"), "{zh}");
    assert!(zh.contains("分组=产品"), "{zh}");
    assert!(zh.contains("无匹配记录"), "{zh}");

    // `search` 与 `list` 共用同一条空结果提示。
    let searched = project.ok(&["search", "查不到的词"]);
    assert_eq!(searched.trim(), "无匹配记录");

    project.write("itemark.toml", &CONFIG.replace("zh-CN", "en"));
    let en = project.ok(&["list", "--group", "产品"]);
    assert!(en.contains("0 record(s)"), "{en}");
    assert!(en.contains("group=产品"), "{en}");
    assert!(en.contains("no matching records"), "{en}");
}

#[test]
fn long_titles_keep_list_and_summary_metadata_on_separate_lines() {
    let project = Project::new();
    project.configure();
    let title = "这是一个特意写得比较长的标题，用于确认列表中元数据不会被挤到标题后面";
    let id = project.add_work(title, "in_progress");

    let listed = project.ok(&["list"]);
    let mut lines = listed.lines();
    assert_eq!(lines.next(), Some("共 1 条记录"));
    assert_eq!(lines.next(), Some("")); // header is separated from the first item
    assert_eq!(lines.next(), Some(format!("{id} {title}").as_str()));
    let metadata = lines.next().expect("metadata line");
    assert!(
        metadata.starts_with("  kind：work · group：产品"),
        "{metadata}"
    );
    assert!(metadata.contains("状态：in_progress"), "{metadata}");

    let summary = project.ok(&["summary"]);
    let summary_lines: Vec<_> = summary.lines().collect();
    assert!(summary_lines[1].starts_with("todo："));
    assert!(summary_lines[2].starts_with("in_progress："), "{summary}");
    assert!(
        summary.contains(&format!("- {id} {title}\n  kind：work\n  group：产品")),
        "{summary}"
    );
}

#[test]
fn id_order_is_numeric_in_queries_and_summaries() {
    let project = Project::new();
    project.configure();
    let ids = ["IM-1", "IM-2", "IM-10", "IM-11"];
    for id in ids {
        project.write(
            &format!("itemark/items/{id}-排序测试.md"),
            &format!(
                "---\nid: {id}\nkind: work\ngroup: 产品\ntitle: 排序测试 {id}\nstatus: in_progress\n---\n\n## 目标\n\n共同匹配词\n\n## 下一步\n\n共同匹配词\n"
            ),
        );
    }

    let list = project.ok(&["list", "--json"]);
    assert_eq!(ids_in(&list), ids);
    let filtered = project.ok(&["list", "--group", "产品", "--json"]);
    assert_eq!(ids_in(&filtered), ids);

    let search = project.ok(&["search", "共同匹配词", "--json"]);
    assert_eq!(ids_in(&search), ids);

    let summary = project.ok(&["summary", "--json"]);
    assert_eq!(json_ids(&summary, "sources"), ids);
    assert_eq!(json_ids(&summary, "items"), ids);
    let handoff = project.ok(&["summary", "--handoff", "--json"]);
    assert_eq!(json_ids(&handoff, "sources"), ids);
    assert_eq!(json_ids(&handoff, "in_progress"), ids);

    let added = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "产品",
        "--set",
        "title=排序后的新事项",
        "--set",
        "status=todo",
        "--json",
    ]);
    assert_eq!(ids_in(&added), ["IM-12"]);
}

#[test]
fn list_displays_status_values_verbatim_and_names_active_filters() {
    let project = Project::new();
    project.configure();
    let id = project.add_work("原始状态筛选", "in_progress");

    let listed = project.ok(&["list", "--status", "in_progress"]);
    assert!(listed.contains("状态=in_progress"), "{listed}");
    assert!(listed.contains(&format!("{id} 原始状态筛选")), "{listed}");
    assert!(listed.contains("状态：in_progress"), "{listed}");
    assert!(listed.contains("共 1 条记录"), "{listed}");

    let shown = project.ok(&["show", &id]);
    assert!(shown.contains("状态：in_progress"), "{shown}");

    let filtered = project.ok(&["list", "--status", "进行中", "--json"]);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&filtered).unwrap()["count"],
        0
    );
}

fn json_ids(json: &str, key: &str) -> Vec<String> {
    serde_json::from_str::<serde_json::Value>(json).expect("valid JSON output")[key]
        .as_array()
        .expect("array of records")
        .iter()
        .map(|value| {
            value
                .as_str()
                .or_else(|| value["id"].as_str())
                .expect("record ID")
                .to_string()
        })
        .collect()
}
