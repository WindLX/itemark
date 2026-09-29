//! `kind list/show/check`。

use std::collections::BTreeSet;

use crate::checks::{CheckReport, Issue, IssueKind};
use crate::error::Result;
use crate::kind::template_body;
use crate::output::{fill, labels, print_json};
use crate::record::markdown::BodyDoc;
use crate::style::{self, paint};
use crate::workspace::config::KindConfig;

use super::Context;
use super::args::{KindCheckArgs, KindShowArgs};
use super::item;

pub fn list(context: &Context) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let config = workspace.config();
    if context.mode.is_json() {
        print_json(&serde_json::json!({
            "kinds": config
                .kinds
                .iter()
                .map(|kind| kind_json(config, kind))
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
                &paint(style::heading(), labels.kinds()),
                &paint(style::accent(), &config.kinds.len().to_string()),
            ]
        )
    );
    for kind in &config.kinds {
        println!(
            "{}",
            fill(
                labels.kind_summary(),
                &[
                    &paint(style::accent(), &kind.name),
                    &kind.fields.len().to_string()
                ]
            )
        );
        if !kind.description.is_empty() {
            println!(
                "  {}",
                fill(labels.line(), &[labels.description(), &kind.description])
            );
        }
    }
    Ok(())
}

pub fn show(context: &Context, args: &KindShowArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let config = workspace.config();

    let kinds: Vec<&KindConfig> = match args.name.as_deref() {
        Some(name) => vec![config.require_kind(name)?],
        None => config.kinds.iter().collect(),
    };

    if context.mode.is_json() {
        let values: Vec<serde_json::Value> =
            kinds.iter().map(|kind| kind_json(config, kind)).collect();
        if args.name.is_some() {
            print_json(&values[0])?;
        } else {
            print_json(&serde_json::json!({ "kinds": values }))?;
        }
        return Ok(());
    }

    let labels = labels(&language);
    for kind in kinds {
        println!(
            "{}",
            fill(
                labels.line(),
                &[
                    &paint(style::heading(), labels.kind()),
                    &paint(style::accent(), &kind.name)
                ]
            )
        );
        if !kind.description.is_empty() {
            println!(
                "{}",
                fill(labels.line(), &[labels.description(), &kind.description])
            );
        }
        if let Some(path) = config.template_path(kind) {
            let presence = if path.is_file() {
                paint(style::ok(), labels.present())
            } else {
                paint(style::warn(), labels.missing())
            };
            println!(
                "{}",
                fill(
                    labels.template_line(),
                    &[
                        labels.template(),
                        &path.display().to_string(),
                        presence.as_str()
                    ]
                )
            );
        }
        if !kind.required_sections.is_empty() {
            println!(
                "{}",
                fill(
                    labels.line(),
                    &[
                        labels.required_sections(),
                        &kind.required_sections.join(labels.list_separator())
                    ]
                )
            );
        }
        if let Some((field, values)) = kind.completion_rule() {
            println!(
                "{}",
                fill(
                    labels.line(),
                    &[
                        labels.completion(),
                        &format!("{field} = {}", values.join("|"))
                    ]
                )
            );
        }
        println!(
            "{}",
            fill(
                labels.heading(),
                &[&paint(style::heading(), labels.fields())]
            )
        );
        for field in &kind.fields {
            let required = if field.required {
                labels.required_marker()
            } else {
                ""
            };
            let values = if field.values.is_empty() {
                String::new()
            } else {
                format!(" [{}]", field.values.join("|"))
            };
            println!(
                "{}",
                fill(
                    labels.field_line(),
                    &[&field.name, &field.field_type, &values, required]
                )
            );
        }
        if let Ok(Some(body)) = template_body(config, kind) {
            println!(
                "{}",
                fill(labels.body_block(), &[labels.body(), body.render().trim()])
            );
        }
    }
    Ok(())
}

pub fn check(context: &Context, args: &KindCheckArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let config = workspace.config();

    let kinds: Vec<&KindConfig> = match args.name.as_deref() {
        Some(name) => vec![config.require_kind(name)?],
        None => config.kinds.iter().collect(),
    };

    // 发现与记录检查同构：同一份 CheckReport，同一处渲染。
    let mut report = CheckReport {
        issues: Vec::new(),
        warnings: Vec::new(),
        checked: 0,
    };
    for kind in kinds {
        report.checked += 1;
        let mut record_issue = |detail: String| {
            report.issues.push(Issue {
                kind: IssueKind::KindDeclaration,
                target: kind.name.clone(),
                detail,
            });
        };

        let completion_field = kind.completion_field.clone();
        let completion_values = kind.completion_values.clone();
        if let Some(field) = completion_field.as_deref() {
            match kind.field(field) {
                None => record_issue(format!(
                    "completion_field `{field}` is not a declared field"
                )),
                Some(declared) => {
                    for value in &completion_values {
                        if let Err(error) = crate::kind::check_field_value(kind, declared, value) {
                            record_issue(error.message().to_string());
                        }
                    }
                }
            }
        }
        let Some(path) = config.template_path(kind) else {
            continue;
        };
        if !path.is_file() {
            record_issue(format!("template is missing: {}", path.display()));
            continue;
        }
        let body: BodyDoc = match template_body(config, kind) {
            Ok(Some(body)) => body,
            Ok(None) => BodyDoc::parse(""),
            Err(error) => {
                record_issue(error.message().to_string());
                continue;
            }
        };
        let titles: BTreeSet<String> = body.titles().into_iter().collect();
        for section in &kind.required_sections {
            if !titles.contains(section) {
                record_issue(format!(
                    "template `{}` has no section `{section}`",
                    path.display()
                ));
            }
        }
    }

    item::print_report(context.mode, &language, item::ReportScope::Kinds, &report)
}

fn kind_json(config: &crate::workspace::Config, kind: &KindConfig) -> serde_json::Value {
    let template = config.template_path(kind);
    serde_json::json!({
        "name": kind.name,
        "description": kind.description,
        "template": template.as_ref().map(|path| path.display().to_string()),
        "template_exists": template.as_ref().is_some_and(|path| path.is_file()),
        "required_sections": kind.required_sections,
        "completion_field": kind.completion_field,
        "completion_values": kind.completion_values,
        "status_field": kind.status_field(),
        "fields": kind.fields.iter().map(|field| serde_json::json!({
            "name": field.name,
            "type": field.field_type,
            "required": field.required,
            "values": field.values,
        })).collect::<Vec<_>>(),
    })
}
