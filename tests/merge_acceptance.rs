//! 聚合预览：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn merge_dry_run_does_not_write_or_consume_an_id() {
    let project = Project::new();
    project.configure();
    let first = project.add_work("来源一", "in_progress");
    let second = project.add_work("来源二", "in_progress");
    let first_path = project.item_file(&first);
    let second_path = project.item_file(&second);
    let first_before = fs::read(&first_path).expect("read first source");
    let second_before = fs::read(&second_path).expect("read second source");

    let preview = project.ok(&[
        "merge",
        &first,
        &second,
        "--title",
        "预览聚合结果",
        "--group",
        "研究",
        "--dry-run",
        "--json",
    ]);

    assert!(
        preview.contains("IM-3"),
        "preview should show a candidate ID: {preview}"
    );
    assert_eq!(
        fs::read(first_path).expect("reread first source"),
        first_before
    );
    assert_eq!(
        fs::read(second_path).expect("reread second source"),
        second_before
    );
    assert!(!project.path().join("itemark-records/archive").exists());

    let next = project.add_work("dry-run 后新建", "todo");
    assert_eq!(next, "IM-3", "dry-run must not reserve an ID");
}

#[test]
fn merge_rejects_conflicting_fields_without_modifying_sources() {
    let project = Project::new();
    project.configure();
    let first = project.add_work("待办来源", "todo");
    let second = project.add_work("进行中来源", "in_progress");
    let first_path = project.item_file(&first);
    let second_path = project.item_file(&second);
    let first_before = fs::read(&first_path).expect("read first source");
    let second_before = fs::read(&second_path).expect("read second source");

    let (_, error) = project.fail(&[
        "merge",
        &first,
        &second,
        "--title",
        "冲突聚合",
        "--group",
        "研究",
    ]);

    assert!(
        error.contains("status"),
        "error should identify the conflicting field: {error}"
    );
    assert!(
        error.contains(&first),
        "error should identify the first source: {error}"
    );
    assert!(
        error.contains(&second),
        "error should identify the second source: {error}"
    );
    assert_eq!(
        fs::read(first_path).expect("reread first source"),
        first_before
    );
    assert_eq!(
        fs::read(second_path).expect("reread second source"),
        second_before
    );
    assert!(!project.path().join("itemark-records/archive").exists());

    let next = project.add_work("冲突后新建", "todo");
    assert_eq!(next, "IM-3", "a rejected merge must not consume an ID");
}

#[test]
fn merge_retargets_references_preserves_source_history_and_clears_review_marker() {
    let project = Project::new();
    project.configure();
    let first = project.add_work("来源一", "in_progress");
    let second = project.add_work("来源二", "in_progress");
    project.ok(&["update", &second, "--group", "研究"]);
    let external_dependency = project.add_work("外部依赖", "todo");
    project.ok(&[
        "update",
        &first,
        "--set",
        &format!("depends_on={external_dependency},{second}"),
    ]);
    project.ok(&[
        "update",
        &second,
        "--set",
        &format!("depends_on={external_dependency},{first}"),
    ]);
    project.ok(&[
        "update",
        &first,
        "--section",
        &format!("目标=保留代码示例 `[[{second}]]`"),
    ]);
    project.ok(&["log", &first, &format!("来源历史保留标记 [[{second}]]")]);

    let ref_first = project.add_note("引用来源一", "研究");
    project.ok(&[
        "update",
        &ref_first,
        "--section",
        &format!("目标=正文引用 [[{first}]]，代码示例 `[[{first}]]`"),
    ]);
    let ref_second = project.add_note("引用来源二", "研究");
    project.ok(&[
        "update",
        &ref_second,
        "--section",
        &format!("目标=正文引用 [[{second}]]"),
    ]);
    let child = project.add_work("依赖来源的子项", "todo");
    project.ok(&[
        "update",
        &child,
        "--set",
        &format!("parent={first}"),
        "--set",
        &format!("depends_on={first},{second}"),
    ]);

    let first_body_before = record(&project.ok(&["show", &first, "--json"]))["body"]
        .as_str()
        .expect("source body")
        .to_string();
    assert!(first_body_before.contains(&format!("来源历史保留标记 [[{second}]]")));

    project.ok(&[
        "merge",
        &first,
        &second,
        "--title",
        "聚合结果",
        "--group",
        "研究",
    ]);
    let listed = record(&project.ok(&["list", "--kind", "work", "--group", "研究", "--json"]));
    let merged = listed["items"]
        .as_array()
        .expect("list items")
        .iter()
        .find(|item| item["title"] == "聚合结果")
        .expect("merged record appears in the target group");
    let merged_id = merged["id"].as_str().expect("merged stable ID").to_string();

    assert!(archived_file(&project, &first).exists());
    assert!(archived_file(&project, &second).exists());
    let merged_detail = record(&project.ok(&["show", &merged_id, "--json"]));
    assert_eq!(merged_detail["kind"], "work");
    assert_eq!(merged_detail["group"], "研究");
    assert_eq!(merged_detail["status"], "in_progress");
    let merged_from = merged_detail["fields"]["merged_from"]
        .as_array()
        .expect("merged_from IDs");
    assert!(merged_from.contains(&serde_json::Value::String(first.clone())));
    assert!(merged_from.contains(&serde_json::Value::String(second.clone())));
    assert_eq!(
        merged_detail["fields"]["depends_on"],
        serde_json::json!([external_dependency])
    );
    assert_eq!(merged_detail["fields"]["needs_review"], true);

    let first_after = record(&project.ok(&["show", &first, "--json"]));
    assert_eq!(first_after["fields"]["merged_into"], merged_id);
    let first_body_after = first_after["body"]
        .as_str()
        .expect("source body after merge");
    assert_eq!(
        first_body_after, first_body_before,
        "source body and history remain unchanged"
    );

    let ref_first_after = record(&project.ok(&["show", &ref_first, "--json"]));
    let ref_first_body = ref_first_after["body"].as_str().expect("reference body");
    assert!(ref_first_body.contains(&format!("正文引用 [[{merged_id}]]")));
    assert!(ref_first_body.contains(&format!("代码示例 `[[{first}]]`")));
    let ref_second_after = record(&project.ok(&["show", &ref_second, "--json"]));
    assert!(
        ref_second_after["body"]
            .as_str()
            .expect("reference body")
            .contains(&format!("正文引用 [[{merged_id}]]"))
    );

    let child_after = record(&project.ok(&["show", &child, "--json"]));
    assert_eq!(child_after["fields"]["parent"], merged_id);
    assert_eq!(
        child_after["fields"]["depends_on"],
        serde_json::json!([merged_id])
    );

    let before_review = record(&project.ok(&["check", &merged_id, "--json"]));
    assert!(
        before_review["warnings"]
            .as_array()
            .is_some_and(|warnings| !warnings.is_empty())
    );
    project.ok(&["update", &merged_id, "--reviewed"]);

    let reviewed = record(&project.ok(&["show", &merged_id, "--json"]));
    assert_eq!(
        reviewed["status"], "in_progress",
        "clearing review must not change business status"
    );
    assert_ne!(reviewed["fields"]["needs_review"], true);
    let after_review = record(&project.ok(&["check", &merged_id, "--json"]));
    assert!(
        after_review["warnings"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
}

fn record(output: &str) -> serde_json::Value {
    serde_json::from_str(output).expect("valid CLI JSON")
}

fn archived_file(project: &Project, id: &str) -> PathBuf {
    let prefix = format!("{id}-");
    fs::read_dir(project.path().join("itemark-records/archive"))
        .expect("archive directory exists")
        .map(|entry| entry.expect("archive entry").path())
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(&prefix))
        })
        .unwrap_or_else(|| panic!("archived record {id} was not found"))
}
