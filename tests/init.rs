//! init：首次初始化项目，--force 只重写配置、不删记录
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn init_writes_the_project_once_and_force_rewrites_it() {
    let project = Project::new();

    project.ok(&["init"]);
    assert!(project.exists("worklog.toml"), "init writes project config");
    assert!(
        project.exists("worklog/items"),
        "init creates the items root"
    );
    assert!(project.exists("worklog/templates"));

    let (_code, output) = project.fail(&["init"]);
    assert!(
        output.contains("worklog.toml"),
        "the error names the existing config: {output}"
    );

    project.ok(&["init", "--force"]);
    assert!(project.read("worklog.toml").contains("zh-CN"));
}

#[test]
fn init_writes_into_an_explicit_project_directory() {
    let project = Project::new();

    project.ok(&["init", "--project", "nested/project", "--root", "docs"]);
    assert!(project.exists("nested/project/worklog.toml"));
    assert!(
        project.exists("nested/project/docs/items"),
        "the root is relative to the config file"
    );
    assert!(
        project
            .read("nested/project/worklog.toml")
            .contains("root = \"docs\"")
    );

    project.ok(&["init", "--project", "nested/project", "--force"]);
    let rewritten = project.read("nested/project/worklog.toml");
    assert!(
        rewritten.contains("root = \"docs\""),
        "an existing project keeps its root unless the CLI overrides it: {rewritten}"
    );
}
