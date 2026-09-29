//! `init` 与项目配置定位。

use std::path::{Path, PathBuf};

use crate::error::{Result, WorkspaceError};
use crate::output::{fill, labels};
use crate::style::{self, paint};
use crate::workspace::config::{Config, DEFAULT_ROOT};

use super::Context;
use super::args::InitArgs;

/// 定位 `itemark.toml`：显式 `--project` 优先，否则从当前目录向上查找。
pub fn find_config(project: Option<&Path>) -> Result<PathBuf> {
    if let Some(project) = project {
        let candidate = project.join(crate::workspace::CONFIG_FILE);
        if !candidate.is_file() {
            return Err(WorkspaceError::usage(format!(
                "no {} in `{}`",
                crate::workspace::CONFIG_FILE,
                project.display()
            ))
            .at(&candidate));
        }
        return Ok(candidate);
    }
    let start = std::env::current_dir().map_err(|error| {
        WorkspaceError::runtime(format!("cannot read current directory: {error}"))
    })?;
    let mut current: Option<&Path> = Some(start.as_path());
    while let Some(directory) = current {
        let candidate = directory.join(crate::workspace::CONFIG_FILE);
        if candidate.is_file() {
            return Ok(candidate);
        }
        current = directory.parent();
    }
    Err(WorkspaceError::usage(format!(
        "no {} found in this directory or its parents; run `itemark init` first",
        crate::workspace::CONFIG_FILE
    )))
}

/// 初始化项目配置与推荐目录；不写入 kind/group 种子数据。
pub fn init(context: &Context, args: &InitArgs) -> Result<()> {
    let project_dir = match context.project.as_deref() {
        Some(project) => project.to_path_buf(),
        None => std::env::current_dir().map_err(|error| {
            WorkspaceError::runtime(format!("cannot read current directory: {error}"))
        })?,
    };
    std::fs::create_dir_all(&project_dir)
        .map_err(|error| WorkspaceError::from(error).at(&project_dir))?;
    let config_path = project_dir.join(crate::workspace::CONFIG_FILE);
    if config_path.exists() && !args.force {
        return Err(WorkspaceError::runtime(format!(
            "{} already exists; pass --force to rewrite it",
            config_path.display()
        ))
        .at(&config_path));
    }

    let existed = config_path.exists();
    let system_language = crate::i18n::system_language();
    let root_override = context.root.as_deref();
    let root = root_override
        .map(|path| path.display().to_string())
        .or_else(|| existing_value(&config_path, "root"))
        .unwrap_or_else(|| DEFAULT_ROOT.to_string());

    if !existed {
        let config = bootstrap_config(system_language, &root);
        std::fs::write(&config_path, config)
            .map_err(|error| WorkspaceError::from(error).at(&config_path))?;
    } else if args.force {
        let mut contents = std::fs::read_to_string(&config_path)
            .map_err(|error| crate::error::read_error(&config_path, error))?;
        let language_exists = existing_value(&config_path, "language").is_some();
        if let Some(root_override) = root_override {
            contents = set_top_level_value(&contents, "root", &root_override.display().to_string());
        }
        if !language_exists {
            contents = set_top_level_value(&contents, "language", system_language);
        }
        if contents != std::fs::read_to_string(&config_path).unwrap_or_default() {
            std::fs::write(&config_path, contents)
                .map_err(|error| WorkspaceError::from(error).at(&config_path))?;
        }
    }

    let config = Config::load(&config_path, None)?;
    for directory in [
        config.items_dir(),
        config.templates_dir(),
        config.summaries_dir(),
    ] {
        std::fs::create_dir_all(&directory)
            .map_err(|error| WorkspaceError::from(error).at(&directory))?;
    }

    ensure_lock_ignored(&config.root)?;
    if !existed {
        write_starter_templates(&config, system_language)?;
    }

    let labels = labels(system_language);
    println!(
        "{}",
        fill(
            labels.init_line(),
            &[
                &paint(style::ok(), labels.created()),
                &paint(style::muted(), &config_path.display().to_string()),
                &config.root.display().to_string()
            ]
        )
    );
    println!(
        "{}",
        fill(labels.next_step_hint(), &[crate::workspace::CONFIG_FILE])
    );
    Ok(())
}

/// Create a small editable starter project for a new configuration.
fn bootstrap_config(language: &str, root: &str) -> String {
    let english = crate::i18n::is_english(language);
    let root = toml::Value::String(root.to_string());
    let general = "general";
    let work_description = if english {
        "Actionable work item"
    } else {
        "可继续推进的工作事项"
    };
    let fact_description = if english {
        "A claim with its source"
    } else {
        "有来源的事实陈述"
    };
    let term_description = if english {
        "A term with a concise definition"
    } else {
        "需要简明定义的术语"
    };
    format!(
        "language = \"{language}\"\nroot = {root}\n\n[[groups]]\nname = \"{general}\"\n\n[[kinds]]\nname = \"work\"\ndescription = \"{work_description}\"\ntemplate = \"templates/work.md\"\nrequired_sections = [{}, {}]\ncompletion_field = \"status\"\ncompletion_values = [\"done\"]\nfields = [\n  {{ name = \"title\", type = \"string\", required = true }},\n  {{ name = \"status\", type = \"enum\", required = true, values = [\"todo\", \"in_progress\", \"blocked\", \"done\"] }},\n  {{ name = \"completion_note\", type = \"string\" }},\n  {{ name = \"completion_evidence\", type = \"string\" }},\n]\n\n[[kinds]]\nname = \"fact\"\ndescription = \"{fact_description}\"\ntemplate = \"templates/fact.md\"\nrequired_sections = [{}, {}]\nfields = [\n  {{ name = \"title\", type = \"string\", required = true }},\n  {{ name = \"verification\", type = \"enum\", required = true, values = [\"unverified\", \"verified\", \"disputed\"] }},\n]\n\n[[kinds]]\nname = \"term\"\ndescription = \"{term_description}\"\ntemplate = \"templates/term.md\"\nrequired_sections = [{}]\nfields = [\n  {{ name = \"title\", type = \"string\", required = true }},\n  {{ name = \"aliases\", type = \"string\" }},\n]\n",
        toml_string(if english { "Goal" } else { "目标" }),
        toml_string(if english { "Acceptance" } else { "验收" }),
        toml_string(if english { "Statement" } else { "事实陈述" }),
        toml_string(if english { "Sources" } else { "来源" }),
        toml_string(if english { "Definition" } else { "定义" }),
    )
}

fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}

fn write_starter_templates(config: &Config, language: &str) -> Result<()> {
    let english = crate::i18n::is_english(language);
    let (goal, acceptance, next, progress, history, statement, sources, definition) = if english {
        (
            "Goal",
            "Acceptance",
            "Next step",
            "Progress",
            "History",
            "Statement",
            "Sources",
            "Definition",
        )
    } else {
        (
            "目标",
            "验收",
            "当前下一步",
            "进展",
            "历史进展",
            "事实陈述",
            "来源",
            "定义",
        )
    };
    let templates = [
        (
            "work.md",
            format!(
                "---\nid: \"{{{{id}}}}\"\nkind: work\ngroup: \"{{{{group}}}}\"\ntitle: \"{{{{title}}}}\"\nstatus: todo\ncompletion_note: \"\"\ncompletion_evidence: \"\"\n---\n\n## {goal}\n\n## {acceptance}\n\n## {next}\n\n## {progress}\n\n## {history}\n"
            ),
        ),
        (
            "fact.md",
            format!(
                "---\nid: \"{{{{id}}}}\"\nkind: fact\ngroup: \"{{{{group}}}}\"\ntitle: \"{{{{title}}}}\"\nverification: unverified\n---\n\n## {statement}\n\n## {sources}\n"
            ),
        ),
        (
            "term.md",
            format!(
                "---\nid: \"{{{{id}}}}\"\nkind: term\ngroup: \"{{{{group}}}}\"\ntitle: \"{{{{title}}}}\"\naliases: []\n---\n\n## {definition}\n"
            ),
        ),
    ];
    for (name, contents) in templates {
        let path = config.templates_dir().join(name);
        if !path.exists() {
            std::fs::write(&path, contents)
                .map_err(|error| WorkspaceError::from(error).at(&path))?;
        }
    }
    Ok(())
}

/// Update only a top-level TOML setting, preserving unrelated user text and formatting.
fn set_top_level_value(source: &str, key: &str, value: &str) -> String {
    let setting = format!("{key} = {}", toml_string(value));
    let mut lines = Vec::new();
    let mut inserted = false;
    let mut in_table = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            if !inserted {
                lines.push(setting.clone());
                inserted = true;
            }
            in_table = true;
        }
        if !in_table
            && trimmed
                .split_once('=')
                .is_some_and(|(name, _)| name.trim() == key)
        {
            lines.push(setting.clone());
            inserted = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !inserted {
        lines.insert(0, setting);
    }
    format!("{}\n", lines.join("\n"))
}

/// 读取已存在配置里的顶层字符串键，供 `init` 沿用取值。
fn existing_value(config_path: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(config_path).ok()?;
    let value: toml::Value = toml::from_str(&text).ok()?;
    value.get(key)?.as_str().map(str::to_string)
}

/// Keep the OS-managed project lock file out of version control without replacing local rules.
fn ensure_lock_ignored(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");
    let mut contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(WorkspaceError::from(error).at(&path)),
    };
    if contents.lines().any(|line| line.trim() == "/.itemark.lock") {
        return Ok(());
    }
    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str("/.itemark.lock\n");
    std::fs::write(&path, contents).map_err(|error| WorkspaceError::from(error).at(&path))
}
