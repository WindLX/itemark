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
            &format!("itemark-records/items/{id}-排序测试.md"),
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

#[test]
fn list_combines_repeated_filters_with_or_and_cross_field_and() {
    let project = Project::new();
    project.configure();
    let config = CONFIG.replace(
        "values = [\"todo\", \"in_progress\", \"blocked\", \"done\"]",
        "values = [\"todo\", \"in_progress\", \"blocked\", \"done\", \"review\"]",
    );
    project.write("itemark.toml", &config);
    let todo = project.add_work("待办工作", "todo");
    let in_progress = project.add_work("进行中工作", "in_progress");
    project.ok(&["update", &in_progress, "--group", "研究"]);
    let blocked = project.add_work("阻塞工作", "blocked");
    let note = project.add_note("研究笔记", "研究");
    let custom_status = project.add_work("自定义状态工作", "review");

    let statuses = project.ok(&[
        "list",
        "--status",
        "todo",
        "--status",
        "in_progress",
        "--json",
    ]);
    assert_eq!(ids_in(&statuses), [todo.as_str(), in_progress.as_str()]);

    let raw_status = project.ok(&["list", "--status", "review", "--json"]);
    assert_eq!(ids_in(&raw_status), [custom_status.as_str()]);

    let groups = project.ok(&["list", "--group", "产品", "--group", "研究", "--json"]);
    assert_eq!(
        ids_in(&groups),
        [
            todo.as_str(),
            in_progress.as_str(),
            blocked.as_str(),
            note.as_str(),
            custom_status.as_str()
        ]
    );

    let kinds = project.ok(&["list", "--kind", "work", "--kind", "project-note", "--json"]);
    assert_eq!(
        ids_in(&kinds),
        [
            todo.as_str(),
            in_progress.as_str(),
            blocked.as_str(),
            note.as_str(),
            custom_status.as_str()
        ]
    );

    let combined = project.ok(&[
        "list",
        "--group",
        "研究",
        "--group",
        "产品",
        "--kind",
        "project-note",
        "--kind",
        "work",
        "--status",
        "todo",
        "--status",
        "in_progress",
        "--status",
        "todo",
        "--json",
    ]);
    assert_eq!(ids_in(&combined), [todo.as_str(), in_progress.as_str()]);

    let text = project.ok(&[
        "list",
        "--group",
        "产品",
        "--group",
        "研究",
        "--group",
        "产品",
        "--status",
        "todo",
        "--status",
        "in_progress",
        "--status",
        "todo",
    ]);
    assert!(text.contains("分组=产品 或 研究"), "{text}");
    assert!(text.contains("状态=todo 或 in_progress"), "{text}");
    assert!(text.contains("共 2 条记录"), "{text}");
    assert_eq!(
        text.lines().nth(1),
        Some("筛选条件：分组=产品 或 研究 且 状态=todo 或 in_progress")
    );

    let comma_is_one_literal_value =
        project.ok(&["list", "--status", "todo,in_progress", "--json"]);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&comma_is_one_literal_value).unwrap()["count"],
        0
    );
}

#[test]
fn list_help_explains_repeatable_or_and_and_filter_values() {
    let project = Project::new();
    project.configure();
    let zh = project.ok(&["list", "--help"]);
    assert!(zh.contains("重复"), "{zh}");
    assert!(zh.contains("OR") && zh.contains("AND"), "{zh}");

    project.write("itemark.toml", &CONFIG.replace("zh-CN", "en"));
    let en = project.ok(&["list", "--help"]);
    assert!(en.to_lowercase().contains("repeat"), "{en}");
    assert!(en.contains("OR") && en.contains("AND"), "{en}");
}

#[test]
fn list_filters_merge_role_reference_health_and_review_state() {
    let project = Project::new();
    project.configure();

    project.write(
        "itemark-records/items/IM-1-有效.md",
        "---\nid: IM-1\nkind: work\ngroup: 产品\ntitle: 有效\nstatus: todo\n---\n\n## 目标\n\n有效记录\n",
    );
    project.write(
        "itemark-records/items/IM-2-已废弃.md",
        "---\nid: IM-2\nkind: work\ngroup: 产品\ntitle: 已废弃\nstatus: todo\ndropped: true\n---\n\n## 目标\n\n废弃目标\n",
    );
    project.write(
        "itemark-records/items/IM-3-source.md",
        "---\nid: IM-3\nkind: work\ngroup: 产品\ntitle: 来源角色\nstatus: todo\nmerged_into: IM-99\nparent: IM-99\n---\n\n## 目标\n\n来源记录\n",
    );
    project.write(
        "itemark-records/items/IM-4-result.md",
        "---\nid: IM-4\nkind: work\ngroup: 研究\ntitle: 结果角色\nstatus: todo\nmerged_from: [IM-1]\n---\n\n## 目标\n\n结果记录\n",
    );
    project.write(
        "itemark-records/items/IM-5-both.md",
        "---\nid: IM-5\nkind: work\ngroup: 产品\ntitle: 双重角色待复核\nstatus: todo\nmerged_into: IM-99\nmerged_from: [IM-1]\nparent: IM-99\nneeds_review: true\n---\n\n## 目标\n\n双重角色记录\n",
    );
    project.write(
        "itemark-records/items/IM-6-warning.md",
        "---\nid: IM-6\nkind: work\ngroup: 产品\ntitle: 引用警告\nstatus: todo\nparent: IM-2\n---\n\n## 目标\n\n引用已废弃记录\n",
    );
    project.write(
        "itemark-records/items/IM-7-reviewed-marker.md",
        "---\nid: IM-7\nkind: work\ngroup: 研究\ntitle: 单独复核标记\nstatus: todo\nneeds_review: true\nparent: IM-1\n---\n\n## 目标\n\n复核标记本身不是引用问题\n",
    );

    let sources = project.ok(&["list", "--merge-role", "source", "--json"]);
    assert_eq!(json_ids(&sources, "items"), ["IM-3", "IM-5"]);
    let results = project.ok(&["list", "--merge-role", "result", "--json"]);
    assert_eq!(json_ids(&results, "items"), ["IM-4", "IM-5"]);
    let no_role = project.ok(&["list", "--merge-role", "none", "--json"]);
    assert_eq!(json_ids(&no_role, "items"), ["IM-1", "IM-6", "IM-7"]);

    let errors = project.ok(&["list", "--reference-health", "error", "--json"]);
    assert_eq!(json_ids(&errors, "items"), ["IM-3", "IM-5"], "{errors}");
    let warnings = project.ok(&["list", "--reference-health", "warning", "--json"]);
    assert_eq!(json_ids(&warnings, "items"), ["IM-6"]);
    let error_or_warning = project.ok(&[
        "list",
        "--reference-health",
        "error",
        "--reference-health",
        "warning",
        "--json",
    ]);
    assert_eq!(
        json_ids(&error_or_warning, "items"),
        ["IM-3", "IM-5", "IM-6"]
    );
    let healthy = project.ok(&["list", "--reference-health", "ok", "--json"]);
    assert_eq!(json_ids(&healthy, "items"), ["IM-1", "IM-4", "IM-7"]);

    let reviewed = project.ok(&["list", "--needs-review", "--json"]);
    assert_eq!(json_ids(&reviewed, "items"), ["IM-5", "IM-7"]);
    let text = project.ok(&[
        "list",
        "--merge-role",
        "source",
        "--merge-role",
        "result",
        "--reference-health",
        "error",
        "--needs-review",
    ]);
    assert!(text.contains("合并角色=source 或 result"), "{text}");
    assert!(text.contains("引用健康度=error"), "{text}");
    assert!(text.contains("需要复核"), "{text}");
    let combined = project.ok(&[
        "list",
        "--merge-role",
        "source",
        "--merge-role",
        "result",
        "--reference-health",
        "error",
        "--needs-review",
        "--json",
    ]);
    assert_eq!(json_ids(&combined, "items"), ["IM-5"]);
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
