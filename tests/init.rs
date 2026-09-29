//! init：首次初始化项目，--force 只重写配置、不删记录
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

#[test]
fn init_writes_the_project_once_and_force_rewrites_it() {
    let project = Project::new();

    project.ok(&["init"]);
    assert!(project.exists("itemark.toml"), "init writes project config");
    assert!(
        project.exists("itemark/items"),
        "init creates the items root"
    );
    assert!(project.exists("itemark/templates"));
    for kind in ["work", "fact", "term"] {
        assert!(project.exists(&format!("itemark/templates/{kind}.md")));
    }

    let (_code, output) = project.fail(&["init"]);
    assert!(
        output.contains("itemark.toml"),
        "the error names the existing config: {output}"
    );

    project.ok(&["init", "--force"]);
    assert!(project.read("itemark.toml").contains("zh-CN"));
}

#[test]
fn init_writes_into_an_explicit_project_directory() {
    let project = Project::new();

    project.ok(&["init", "--project", "nested/project", "--root", "docs"]);
    assert!(project.exists("nested/project/itemark.toml"));
    assert!(
        project.exists("nested/project/docs/items"),
        "the root is relative to the config file"
    );
    assert!(
        project
            .read("nested/project/itemark.toml")
            .contains("root = \"docs\"")
    );

    project.ok(&["init", "--project", "nested/project", "--force"]);
    let rewritten = project.read("nested/project/itemark.toml");
    assert!(
        rewritten.contains("root = \"docs\""),
        "an existing project keeps its root unless the CLI overrides it: {rewritten}"
    );
}

#[test]
fn init_preserves_existing_gitignore_and_adds_lock_entry_once() {
    let project = Project::new();
    project.write("itemark/.gitignore", "# keep this rule\nitems/*.tmp");
    project.ok(&["init"]);
    let first = project.read("itemark/.gitignore");
    assert!(
        first.starts_with("# keep this rule\nitems/*.tmp\n"),
        "{first}"
    );
    assert_eq!(first.matches("/.itemark.lock").count(), 1, "{first}");

    project.ok(&["init", "--force"]);
    let second = project.read("itemark/.gitignore");
    assert_eq!(second.matches("/.itemark.lock").count(), 1, "{second}");
}

#[test]
fn new_project_gets_editable_work_fact_and_term_starters() {
    let project = Project::new();
    let output = project
        .command()
        .env("LANG", "zh_CN.UTF-8")
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .arg("init")
        .output()
        .expect("run init");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config = project.read("itemark.toml");
    assert!(config.contains("name = \"general\""), "{config}");
    for kind in ["work", "fact", "term"] {
        assert!(config.contains(&format!("name = \"{kind}\"")), "{config}");
        assert!(project.exists(&format!("itemark/templates/{kind}.md")));
    }

    let work_output = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "general",
        "--set",
        "title=起步事项",
        "--json",
    ]);
    let work_json: serde_json::Value = serde_json::from_str(&work_output).expect("valid JSON");
    let work = work_json["id"].as_str().expect("work ID");
    assert_eq!(work_json["status"], "todo");
    let shown = project.ok(&["show", work]);
    assert!(!shown.contains("completion_note："), "{shown}");
    assert!(!shown.contains("completion_evidence："), "{shown}");
    let (_, missing) = project.fail(&["check", work]);
    assert!(missing.contains("目标"), "{missing}");
    assert!(missing.contains("验收"), "{missing}");
    project.ok(&[
        "update",
        work,
        "--section",
        "目标=完成初始目标",
        "--section",
        "验收=检查并通过",
    ]);
    let (_, incomplete) = project.fail(&["update", work, "--set", "status=done"]);
    assert!(incomplete.contains("完成说明"), "{incomplete}");
    assert!(incomplete.contains("证据"), "{incomplete}");
    project.ok(&[
        "update",
        work,
        "--set",
        "status=done",
        "--set",
        "completion_note=已经完成",
        "--set",
        "completion_evidence=手工验收",
    ]);
    project.ok(&["check", work]);

    let fact_output = project.ok(&[
        "add",
        "--kind",
        "fact",
        "--group",
        "general",
        "--set",
        "title=事实",
        "--json",
    ]);
    let fact_json: serde_json::Value = serde_json::from_str(&fact_output).expect("valid JSON");
    assert_eq!(fact_json["fields"]["verification"], "unverified");
    let fact = fact_json["id"].as_str().expect("fact ID");
    let (_, fact_missing) = project.fail(&["check", fact]);
    assert!(fact_missing.contains("事实陈述"), "{fact_missing}");
    assert!(fact_missing.contains("来源"), "{fact_missing}");

    let term_output = project.ok(&[
        "add",
        "--kind",
        "term",
        "--group",
        "general",
        "--set",
        "title=术语",
        "--json",
    ]);
    let term_json: serde_json::Value = serde_json::from_str(&term_output).expect("valid JSON");
    assert_eq!(term_json["fields"]["aliases"], serde_json::json!([]));
    let term = term_json["id"].as_str().expect("term ID");
    let (_, term_missing) = project.fail(&["check", term]);
    assert!(term_missing.contains("定义"), "{term_missing}");
}

#[test]
fn force_init_preserves_existing_config_and_does_not_inject_starters() {
    let project = Project::new();
    let config = r#"language = "en"
root = "records"

[[groups]]
name = "research"
"#;
    project.write("itemark.toml", config);
    project.ok(&["init", "--force"]);
    assert_eq!(project.read("itemark.toml"), config);
    assert!(project.exists("records/items"));
    assert!(!project.exists("records/templates/work.md"));
}

#[test]
fn english_init_templates_keep_their_validation_names_after_language_changes() {
    let project = Project::new();
    let init = project
        .command()
        .env("LANG", "en_US.UTF-8")
        .env_remove("LANGUAGE")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .arg("init")
        .output()
        .expect("run English init");
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let config = project.read("itemark.toml");
    assert!(config.contains("language = \"en\""), "{config}");
    assert!(
        config.contains("required_sections = [\"Goal\", \"Acceptance\"]"),
        "{config}"
    );
    assert!(
        project
            .read("itemark/templates/work.md")
            .contains("## Goal")
    );

    let added = project.ok(&[
        "add",
        "--kind",
        "work",
        "--group",
        "general",
        "--set",
        "title=English starter",
        "--json",
    ]);
    let value: serde_json::Value = serde_json::from_str(&added).expect("valid JSON");
    let id = value["id"].as_str().expect("record ID");
    let (_, report) = project.fail(&["check", id]);
    assert!(report.contains("Goal"), "{report}");
    assert!(report.contains("Acceptance"), "{report}");

    project.write(
        "itemark.toml",
        &config.replace(r#"language = "en""#, r#"language = "zh-CN""#),
    );
    let (_, translated_report) = project.fail(&["check", id]);
    assert!(translated_report.contains("Goal"), "{translated_report}");
    assert!(
        translated_report.contains("必填分节"),
        "{translated_report}"
    );
}
