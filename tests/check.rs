//! check 与总览：报告缺失分节、证据缺失与「完成但未验证」
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn check_reports_missing_sections_and_completion_evidence() {
    let project = Project::new();
    project.configure();
    let id = project.add_note("空目标", "研究");

    let (code, output) = project.fail(&["check", &id]);
    assert_eq!(
        code, 1,
        "a record with an empty required section fails check"
    );
    assert!(output.contains("目标"), "{output}");

    project.ok(&["update", &id, "--section", "目标=有内容了"]);
    project.ok(&["check", &id]);

    // 手工把一条 work 记录标成完成却删掉证据，check 必须报告。
    let work = project.add_work("手工完成", "todo");
    project.ok(&["update", &work, "--section", "目标=做完了"]);
    project.ok(&["check", &work]);

    let path = format!("worklog/items/{work}.md");
    let hand_edited = project
        .read(&path)
        .replace("status: \"todo\"", "status: \"done\"")
        .replace("status: todo", "status: done");
    project.write(&path, &hand_edited);

    let (code, output) = project.fail(&["check", &work]);
    assert_eq!(
        code, 1,
        "a completion without evidence fails check: {output}"
    );
    assert!(
        output.contains("证据") || output.contains("完成说明"),
        "check names the missing completion evidence: {output}"
    );
}

#[test]
fn unverified_completion_is_counted_separately_in_the_overview() {
    let project = Project::new();
    project.configure();
    project.add_work("进行中", "in_progress");
    project.add_work("阻塞中", "blocked");
    project.add_work("待办", "todo");
    project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=完成未验证",
        "--set",
        "status=done",
        "--set",
        "completion_note=未验证：还没跑端到端",
    ]);

    let overview = project.ok(&["summary", "--json"]);
    assert!(overview.contains("\"in_progress\": 1"), "{overview}");
    assert!(overview.contains("\"blocked\": 1"), "{overview}");
    assert!(overview.contains("\"todo\": 1"), "{overview}");
    assert!(overview.contains("\"done_unverified\": 1"), "{overview}");

    let text = project.ok(&["summary"]);
    assert!(
        text.contains("WL-"),
        "the overview names its sources: {text}"
    );
}

#[test]
fn check_requires_only_the_fields_the_kind_declares() {
    let project = Project::new();
    project.write(
        "worklog.toml",
        r#"
language = "zh-CN"
root = "worklog"

[[groups]]
name = "研究"

[[kinds]]
name = "note"
description = "标题不必填的事实记录"
template = "templates/note.md"
required_sections = ["目标"]
fields = [
  { name = "title", type = "string" },
]

[[kinds]]
name = "task"
description = "标题必填的工作事项"
template = "templates/task.md"
required_sections = ["目标"]
fields = [
  { name = "title", type = "string", required = true },
]
"#,
    );
    project.write(
        "worklog/templates/note.md",
        "---\nid: \"{{id}}\"\nkind: \"note\"\ngroup: \"{{group}}\"\n---\n## 目标\n\n",
    );
    project.write(
        "worklog/templates/task.md",
        "---\nid: \"{{id}}\"\nkind: \"task\"\ngroup: \"{{group}}\"\ntitle: \"{{title}}\"\n---\n## 目标\n\n",
    );
    project.write(
        "worklog/items/WL-0001.md",
        "---\nid: \"WL-0001\"\nkind: \"note\"\ngroup: \"研究\"\n---\n\n## 目标\n\n没有标题也算完整。\n",
    );
    project.write(
        "worklog/items/WL-0002.md",
        "---\nid: \"WL-0002\"\nkind: \"task\"\ngroup: \"研究\"\n---\n\n## 目标\n\n缺了必填标题。\n",
    );

    project.ok(&["check", "WL-0001"]);

    let (code, output) = project.fail(&["check", "WL-0002"]);
    assert_eq!(code, 1, "a declared required field is reported: {output}");
    assert!(output.contains("title"), "{output}");
}
