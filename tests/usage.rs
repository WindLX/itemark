//! 用法错误：未声明字段 / group 与未知 ID 走退出码 2
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn unsupported_fields_and_unknown_ids_are_usage_errors() {
    let project = Project::new();
    project.configure();

    let (code, _) = project.fail(&[
        "add",
        "--kind",
        "project-note",
        "--group",
        "研究",
        "--set",
        "nope=1",
    ]);
    assert_eq!(code, 2, "an undeclared field is a usage error");

    let (code, _) = project.fail(&[
        "add",
        "--kind",
        "project-note",
        "--group",
        "不存在",
        "--set",
        "title=x",
    ]);
    assert_eq!(code, 2, "an undeclared group is a usage error");

    let (code, _) = project.fail(&["show", "WL-9999"]);
    assert_eq!(code, 2, "an unknown ID is a usage error");
}

#[test]
fn references_must_point_at_existing_records() {
    let project = Project::new();
    project.configure();
    project.add_work("已存在的事项", "todo");

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=悬空父项",
        "--set",
        "status=todo",
        "--set",
        "parent=WL-9999",
    ]);
    assert_eq!(code, 2, "an unknown parent is refused: {output}");

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=悬空依赖",
        "--set",
        "status=todo",
        "--set",
        "depends_on=WL-9999",
    ]);
    assert_eq!(code, 2, "an unknown dependency is refused: {output}");

    let (code, output) = project.fail(&["update", "WL-0001", "--set", "parent=WL-0001"]);
    assert_eq!(code, 2, "a record cannot depend on itself: {output}");
}

#[test]
fn depends_on_takes_a_list_and_parent_stays_single() {
    let project = Project::new();
    project.configure();
    let first = project.add_work("前置一", "todo");
    let second = project.add_work("前置二", "todo");

    let added = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=依赖两项",
        "--set",
        "status=todo",
        "--set",
        &format!("depends_on=[{first}, {second}]"),
        "--json",
    ]);
    assert!(
        added.contains(&first) && added.contains(&second),
        "a bracketed list is written as several dependencies: {added}"
    );

    let (code, output) = project.fail(&[
        "add",
        "--kind",
        "work",
        "--group",
        "研究",
        "--set",
        "title=父项过多",
        "--set",
        "status=todo",
        "--set",
        &format!("parent={first},{second}"),
    ]);
    assert_eq!(code, 2, "`parent` takes a single record ID: {output}");
}
