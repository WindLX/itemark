//! `group list/add/show`。

use crate::error::{Result, WorkspaceError};
use crate::output::{fill, labels, print_json};
use crate::style::{self, paint};
use crate::workspace::config::CONFIG_FILE;

use super::Context;
use super::args::{GroupAddArgs, GroupShowArgs};

pub fn list(context: &Context) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let config = workspace.config();

    let groups: Vec<serde_json::Value> = config
        .groups
        .iter()
        .map(|group| {
            let records: Vec<&crate::record::Record> = workspace.index().by_group(group);
            let mut kinds: Vec<String> = records
                .iter()
                .filter_map(|record| record.kind().ok())
                .map(str::to_string)
                .collect();
            kinds.sort();
            kinds.dedup();
            serde_json::json!({
                "name": group,
                "items": records.len(),
                "kinds": kinds,
            })
        })
        .collect();

    if context.mode.is_json() {
        print_json(&serde_json::json!({ "groups": groups }))?;
        return Ok(());
    }
    let labels = labels(&language);
    println!(
        "{}",
        fill(
            labels.line(),
            &[
                &paint(style::heading(), labels.groups()),
                &paint(style::accent(), &config.groups.len().to_string()),
            ]
        )
    );
    for group in &config.groups {
        let records = workspace.index().by_group(group);
        println!(
            "{}",
            fill(
                labels.group_summary(),
                &[&paint(style::accent(), group), &records.len().to_string(),]
            )
        );
    }
    Ok(())
}

pub fn add(context: &Context, args: &GroupAddArgs) -> Result<()> {
    let workspace = context.locked_workspace()?;
    let language = workspace.project_language();
    let config = workspace.config();
    if config.has_group(&args.name) {
        return Err(WorkspaceError::usage(format!(
            "group `{}` is already declared",
            args.name
        )));
    }
    if config.kinds.is_empty() && config.groups.is_empty() && config.raw_text.is_empty() {
        return Err(WorkspaceError::runtime(format!(
            "no {CONFIG_FILE} in this project; run `itemark init` first"
        ))
        .at(&config.config_path));
    }
    validate_group_name(&args.name, &config.config_path)?;

    let mut text = config.raw_text.clone();
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if !text.is_empty() && !text.ends_with("\n\n") {
        text.push('\n');
    }
    text.push_str("[[groups]]\n");
    text.push_str(&format!("name = \"{}\"\n", args.name));
    std::fs::write(&config.config_path, &text)
        .map_err(|error| WorkspaceError::from(error).at(&config.config_path))?;

    if context.mode.is_json() {
        print_json(&serde_json::json!({
            "name": args.name,
            "created": true,
            "path": config.config_path.display().to_string(),
        }))?;
        return Ok(());
    }
    let labels = labels(&language);
    println!(
        "{}",
        fill(
            labels.group_created(),
            &[
                &paint(style::ok(), labels.created()),
                &paint(style::accent(), &args.name),
                &paint(style::muted(), &config.config_path.display().to_string())
            ]
        )
    );
    Ok(())
}

pub fn show(context: &Context, args: &GroupShowArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let config = workspace.config();
    if !config.has_group(&args.name) {
        return Err(WorkspaceError::usage(format!(
            "unknown group `{}`; declared groups: {}",
            args.name,
            if config.groups.is_empty() {
                "none declared".to_string()
            } else {
                config.groups.join(", ")
            }
        )));
    }
    let records: Vec<&crate::record::Record> = workspace.index().by_group(&args.name);
    let mut kinds: Vec<String> = records
        .iter()
        .filter_map(|record| record.kind().ok())
        .map(str::to_string)
        .collect();
    kinds.sort();
    kinds.dedup();

    if context.mode.is_json() {
        print_json(&serde_json::json!({
            "name": args.name,
            "kinds": kinds,
            "items": records
                .iter()
                .map(|record| crate::cli::item::record_json(config, record))
                .collect::<Vec<_>>(),
        }))?;
        return Ok(());
    }
    let labels = labels(&language);
    println!(
        "{}",
        fill(
            labels.line(),
            &[
                &paint(style::heading(), labels.group()),
                &paint(style::accent(), &args.name)
            ]
        )
    );
    println!(
        "{}",
        fill(
            labels.line(),
            &[
                &paint(style::label(), labels.items()),
                &records.len().to_string()
            ]
        )
    );
    if !kinds.is_empty() {
        println!(
            "{}",
            fill(
                labels.line(),
                &[labels.kinds(), &kinds.join(labels.list_separator())]
            )
        );
    }
    for record in records {
        println!(
            "{}",
            crate::cli::item::record_line(&language, config, record)
        );
    }
    Ok(())
}

fn validate_group_name(name: &str, path: &std::path::Path) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceError::usage("group name must not be empty"));
    }
    if trimmed != name {
        return Err(WorkspaceError::usage(format!(
            "group name `{name}` must not start or end with whitespace"
        )));
    }
    if name.contains(['"', '\n', '\r', '\\']) {
        return Err(WorkspaceError::usage(format!(
            "group name `{name}` must not contain quotes, backslashes or line breaks"
        ))
        .at(path));
    }
    Ok(())
}
