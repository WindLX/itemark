//! 项目语言：面向使用者的文案来自项目配置；系统语言只用于 init 与无项目场景。
//!
//! 测试 seam：真实 CLI 进程 + 临时项目目录。

mod common;

use common::*;

fn has_han(text: &str) -> bool {
    text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

fn set_language(project: &Project, language: &str) {
    project.write("itemark.toml", &CONFIG.replace("zh-CN", language));
}

#[test]
fn project_configuration_controls_help_and_record_text() {
    let project = Project::new();
    project.configure();
    set_language(&project, "en");

    let help = project.ok(&["--help"]);
    assert!(help.contains("Usage:"), "{help}");
    assert!(help.contains("Print help"), "{help}");
    assert!(!has_han(&help), "{help}");

    let id = project.add_work("cross-language", "todo");
    let record = project.ok(&["show", &id]);
    assert!(record.contains("title: cross-language"), "{record}");
    assert!(!record.contains('：'), "{record}");
}

#[test]
fn language_override_option_is_removed() {
    let project = Project::new();
    let (code, output) = project.fail(&["--language", "en", "init"]);
    assert_eq!(code, 2);
    assert!(output.contains("不接受参数"), "{output}");
}

#[test]
fn init_selects_a_supported_system_language_and_persists_it() {
    let project = Project::new();
    let output = project.ok(&["init"]);
    let config = project.read("itemark.toml");
    let selected = if config.contains("language = \"en\"") {
        assert!(output.contains("created"), "{output}");
        "en"
    } else {
        assert!(config.contains("language = \"zh-CN\""), "{config}");
        assert!(output.contains("已创建"), "{output}");
        "zh-CN"
    };
    assert!(!selected.is_empty());
}

#[test]
fn init_language_overrides_existing_environment_and_writes_the_choice() {
    #[cfg(unix)]
    {
        let project = Project::new();
        let output = project
            .command()
            .env("LANG", "en_US.UTF-8")
            .env_remove("LANGUAGE")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .arg("init")
            .output()
            .expect("run init with English system locale");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("created"));
        assert!(project.read("itemark.toml").contains("language = \"en\""));
    }
}

#[test]
fn configured_language_wins_over_environment_and_json_keys_stay_stable() {
    let project = Project::new();
    project.configure();
    set_language(&project, "en");
    let add = project
        .command()
        .env("LANG", "zh_CN.UTF-8")
        .args([
            "add",
            "--kind",
            "work",
            "--group",
            "产品",
            "--set",
            "title=stable",
            "--set",
            "status=todo",
            "--json",
        ])
        .output()
        .expect("run add");
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&add.stdout).expect("valid JSON");
    assert_eq!(value["kind"], "work");
    assert_eq!(value["title"], "stable");
    assert!(value["id"].as_str().is_some_and(|id| id.starts_with("IM-")));
    assert!(!String::from_utf8_lossy(&add.stdout).contains('\u{1b}'));
}

#[test]
fn no_config_uses_system_language_without_crashing() {
    #[cfg(unix)]
    {
        let project = Project::new();
        let output = project
            .command()
            .env("LANG", "en_US.UTF-8")
            .env_remove("LANGUAGE")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .arg("--help")
            .output()
            .expect("run help");
        assert!(output.status.success());
        let help = String::from_utf8_lossy(&output.stdout);
        assert!(help.contains("Usage:"), "{help}");
        assert!(help.contains("Print help"), "{help}");

        let error = project
            .command()
            .env("LANG", "zh_CN.UTF-8")
            .env_remove("LANGUAGE")
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .args(["show", "WL-9999"])
            .output()
            .expect("run without config");
        assert!(!error.status.success());
        let error_text = String::from_utf8_lossy(&error.stderr);
        assert!(
            error_text.contains("当前目录及其父目录中没有"),
            "{error_text}"
        );
    }
}

#[test]
fn usage_and_record_errors_use_the_project_language() {
    let project = Project::new();
    project.configure();
    let (_, missing_item) = project.fail(&["show", "WL-9999"]);
    assert!(missing_item.contains("未知事项"), "{missing_item}");
    assert!(
        !missing_item.contains("unknown Itemark item"),
        "{missing_item}"
    );

    let (_, bad_color) = project.fail(&["--color", "purple", "init"]);
    assert!(bad_color.contains("颜色仅支持"), "{bad_color}");
    assert!(!bad_color.contains("expected one of"), "{bad_color}");
}

#[test]
fn check_report_details_are_localized_without_changing_check_behavior() {
    let project = Project::new();
    project.configure();
    let output = project.ok(&["add", "--kind", "work", "--group", "产品", "--json"]);
    let value: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
    let id = value["id"].as_str().expect("record ID");
    let (code, report) = project.fail(&["check", id]);
    assert_eq!(code, 1);
    assert!(report.contains("必填字段"), "{report}");
    assert!(report.contains("必填分节"), "{report}");
    assert!(!report.contains("required field"), "{report}");
    assert!(!report.contains("required section"), "{report}");
}
