//! 业务状态与完成依据只有一个判定：`show` 与 `summary` 必须给出同一个状态
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use std::collections::HashMap;

use common::*;

/// 一个声明了业务状态字段、但没有声明完成字段与完成值的 kind。
const PLAIN_CONFIG: &str = r#"
language = "zh-CN"
root = "itemark-records"

[[groups]]
name = "产品"

[[kinds]]
name = "plain"
description = "有业务状态但没有完成映射"
template = "templates/plain.md"
required_sections = ["目标"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "status", type = "enum", required = true, values = ["todo", "in_progress", "blocked", "done"] },
]
"#;

const PLAIN_TEMPLATE: &str = "---\nid: \"{{id}}\"\nkind: \"plain\"\ngroup: \"{{group}}\"\ntitle: \"{{title}}\"\nstatus: \"{{status}}\"\n---\n## 目标\n\n";

fn summary_states(project: &Project) -> HashMap<String, String> {
    let json: serde_json::Value =
        serde_json::from_str(&project.ok(&["summary", "--json"])).expect("summary json");
    json["items"]
        .as_array()
        .expect("items array")
        .iter()
        .map(|item| {
            (
                item["id"].as_str().expect("item id").to_string(),
                item["state"].as_str().expect("item state").to_string(),
            )
        })
        .collect()
}

fn shown_state(project: &Project, id: &str) -> String {
    let json: serde_json::Value =
        serde_json::from_str(&project.ok(&["show", id, "--json"])).expect("show json");
    json["state"]
        .as_str()
        .expect("show reports the business state")
        .to_string()
}

#[test]
fn show_and_summary_agree_on_the_business_state() {
    let project = Project::new();
    project.configure();

    let todo = project.add_work("待办的工作", "todo");

    let verified = project.add_work("已完成的工作", "todo");
    project.ok(&[
        "update",
        verified.as_str(),
        "--set",
        "status=done",
        "--set",
        "completion_note=已交付并通过复现",
        "--set",
        "completion_evidence=tests/status.rs",
    ]);

    let unverified = project.add_work("完成但缺证据的工作", "todo");
    project.ok(&[
        "update",
        unverified.as_str(),
        "--set",
        "status=done",
        "--set",
        "completion_note=未验证：尚未复现",
    ]);

    let note = project.add_note("一条没有业务状态的笔记", "研究");

    let states = summary_states(&project);
    for (id, expected) in [
        (&todo, "todo"),
        (&verified, "done"),
        (&unverified, "done_unverified"),
        (&note, "none"),
    ] {
        assert_eq!(shown_state(&project, id), expected, "show for {id}");
        assert_eq!(
            states.get(id).map(String::as_str),
            Some(expected),
            "summary for {id}"
        );
    }
}

#[test]
fn a_status_field_without_a_completion_rule_does_not_fork() {
    let project = Project::new();
    project.write("itemark.toml", PLAIN_CONFIG);
    project.write("itemark-records/templates/plain.md", PLAIN_TEMPLATE);

    let id = ids_in(&project.ok(&[
        "add",
        "--kind",
        "plain",
        "--group",
        "产品",
        "--set",
        "title=已完成但没有完成映射",
        "--set",
        "status=done",
        "--json",
    ]))
    .first()
    .cloned()
    .expect("add returns an ID");

    assert_eq!(shown_state(&project, &id), "done");
    assert_eq!(
        summary_states(&project).get(&id).map(String::as_str),
        Some("done")
    );
}
