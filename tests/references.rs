//! 正文稳定引用与结构关系引用：真实 CLI + 临时项目目录。

mod common;

use common::*;

fn report(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("check emits JSON")
}

#[test]
fn missing_body_reference_reports_source_line_and_target() {
    let project = Project::new();
    project.configure();
    let source = project.add_note("含断链的记录", "研究");
    let body = "第一行\n第二行引用 [[IM-404|缺失目标]]";
    project.ok(&["update", &source, "--section", &format!("目标={body}")]);

    let output = project.run(&["check", &source, "--json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let json = report(&output);
    assert_eq!(json["ok"], false, "{json}");
    let issue = json["issues"][0].clone();
    assert_eq!(issue["type"], "broken_reference", "{json}");
    assert_eq!(issue["target"], source, "{json}");
    assert!(
        issue["detail"].as_str().unwrap().contains("IM-404"),
        "{json}"
    );
    assert!(
        issue["detail"].as_str().unwrap().contains("正文第 4 行"),
        "{json}"
    );
}

#[test]
fn valid_references_are_checked_but_code_examples_are_ignored() {
    let project = Project::new();
    project.configure();
    let target = project.add_note("被引用记录", "研究");
    let source = project.add_note("代码示例", "研究");
    let body = format!(
        "正常引用 [[{target}]] 与 [[{target}|显示文字]]。\n\n`[[IM-997]]`\n\n```text\n[[IM-998]]\n```"
    );
    project.ok(&["update", &source, "--section", &format!("目标={body}")]);

    let output = project.run(&["check", &source, "--json"]);
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let json = report(&output);
    assert_eq!(json["issues"].as_array().unwrap().len(), 0, "{json}");
    assert_eq!(json["warnings"].as_array().unwrap().len(), 0, "{json}");
}

#[test]
fn dropped_body_target_is_a_warning_and_check_stays_successful() {
    let project = Project::new();
    project.configure();
    let target = project.add_note("以后不再维护", "研究");
    let source = project.add_note("保留历史引用", "研究");
    project.ok(&[
        "update",
        &source,
        "--section",
        &format!("目标=参考 [[{target}|旧记录]]"),
    ]);
    project.ok(&["drop", &target, "--reason", "已废弃"]);

    let output = project.run(&["check", &source, "--json"]);
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let json = report(&output);
    assert_eq!(json["ok"], true, "{json}");
    assert_eq!(json["issues"].as_array().unwrap().len(), 0, "{json}");
    assert_eq!(
        json["warnings"][0]["type"], "deprecated_reference",
        "{json}"
    );
    assert_eq!(json["warnings"][0]["target"], source, "{json}");
    assert!(
        json["warnings"][0]["detail"]
            .as_str()
            .unwrap()
            .contains(&target),
        "{json}"
    );
}

#[test]
fn dropped_parent_and_dependency_are_warnings_not_errors() {
    let project = Project::new();
    project.configure();
    let target = project.add_note("已废弃父项", "研究");
    let source = project.add_note("引用其结构关系", "研究");
    project.ok(&["update", &source, "--section", "目标=已有目标"]);
    project.ok(&[
        "update",
        &source,
        "--set",
        &format!("parent={target}"),
        "--set",
        &format!("depends_on={target}"),
    ]);
    project.ok(&["drop", &target, "--reason", "已废弃"]);

    let output = project.run(&["check", &source, "--json"]);
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let json = report(&output);
    assert_eq!(json["issues"].as_array().unwrap().len(), 0, "{json}");
    let warnings = json["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 2, "{json}");
    let details = warnings
        .iter()
        .filter_map(|warning| warning["detail"].as_str())
        .collect::<Vec<_>>();
    assert!(
        details.iter().any(|detail| detail.contains("父项")),
        "{json}"
    );
    assert!(
        details.iter().any(|detail| detail.contains("前置依赖")),
        "{json}"
    );
}

#[test]
fn missing_and_self_referential_structure_targets_remain_errors() {
    let project = Project::new();
    project.configure();
    let source = project.add_note("损坏的关系", "研究");
    project.ok(&["update", &source, "--section", "目标=已有目标"]);
    let path = project.item_file(&source);
    let original = fs::read_to_string(&path).expect("read source record");

    let missing = original.replacen("---\n", "---\nparent: IM-404\ndepends_on: [IM-405]\n", 1);
    fs::write(&path, missing).expect("write missing relationship targets");
    let output = project.run(&["check", &source, "--json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let json = report(&output);
    let details = json["issues"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|issue| issue["detail"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(details.len(), 2, "{json}");
    assert!(
        details.iter().any(|detail| detail.contains("IM-404")),
        "{json}"
    );
    assert!(
        details.iter().any(|detail| detail.contains("IM-405")),
        "{json}"
    );

    let self_reference = original.replacen("---\n", &format!("---\nparent: {source}\n"), 1);
    fs::write(&path, self_reference).expect("write self-referential target");
    let output = project.run(&["check", &source, "--json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let json = report(&output);
    assert!(
        json["issues"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("must not reference the record itself"),
        "{json}"
    );
}
