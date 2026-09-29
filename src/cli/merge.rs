//! 合并命令：只读预览与锁内重建/写入。

use crate::checks::{CheckReport, check_record};
use crate::error::{Result, WorkspaceError};
use crate::output::{OutputMode, fill, print_json};
use crate::record::Record;
use crate::workspace::merge::{MergeConflict, MergeOptions, MergePlan, apply, prepare};

use super::Context;
use super::args::MergeArgs;

pub fn merge(context: &Context, args: &MergeArgs) -> Result<()> {
    let options = options(args);
    if args.dry_run {
        let workspace = context.workspace()?;
        let id = workspace.index().next_id()?;
        let plan = prepare(
            workspace.config(),
            workspace.index().records(),
            &id,
            &options,
        )?;
        return print_preview(
            context.mode,
            &workspace.project_language(),
            &plan,
            workspace.index().records(),
        );
    }

    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language();
    let outcome = workspace.transaction(|transaction| {
        let records = transaction.records().to_vec();
        let id = transaction.allocate_id()?;
        let plan = prepare(transaction.config(), &records, &id, &options)?;
        if !plan.is_ready() {
            return Err(WorkspaceError::usage(conflict_message(
                &plan.conflicts,
                &language,
            )));
        }

        validate_plan(transaction.config(), &plan, &records, &language)?;
        apply(transaction, &plan, &records)
    })?;

    if context.mode.is_json() {
        print_json(&serde_json::json!({
            "id": outcome.id,
            "merged_from": outcome.archived_sources,
            "redirected_records": outcome.redirected_records,
            "needs_review": true,
        }))
    } else {
        println!(
            "{}",
            fill(
                crate::i18n::text("merge_complete", &language),
                &[
                    &outcome.id,
                    &outcome.redirected_records.len().to_string(),
                    &outcome.archived_sources.len().to_string(),
                ],
            )
        );
        println!(
            "{}",
            fill(
                crate::i18n::text("merge_redirected", &language),
                &[&localized_ids(&outcome.redirected_records, &language)],
            )
        );
        println!(
            "{}",
            fill(
                crate::i18n::text("merge_archived", &language),
                &[&localized_ids(&outcome.archived_sources, &language)],
            )
        );
        Ok(())
    }
}

fn options(args: &MergeArgs) -> MergeOptions {
    let parent = if args.no_parent {
        Some(None)
    } else {
        args.parent.clone().map(Some)
    };
    MergeOptions {
        source_ids: args.ids.clone(),
        title: args.title.clone(),
        group: args.group.clone(),
        set: args.set.clone(),
        parent,
        completion_note: args.completion_note.clone(),
        completion_evidence: args.completion_evidence.clone(),
    }
}

fn validate_plan(
    config: &crate::workspace::Config,
    plan: &MergePlan,
    records: &[Record],
    language: &str,
) -> Result<()> {
    let known = records
        .iter()
        .filter_map(|record| record.id().ok().map(|id| (id.to_string(), record)))
        .collect::<std::collections::BTreeMap<_, _>>();
    // The candidate ID is intentionally absent unless some invalid source relation points to it.
    let mut report = CheckReport {
        checked: 1,
        ..CheckReport::default()
    };
    check_record(
        config,
        &plan.record,
        &plan.id,
        &known,
        &mut report.issues,
        &mut report.warnings,
    );
    if !report.issues.is_empty() {
        let messages = report
            .issues
            .iter()
            .map(|issue| crate::i18n::localize_diagnostic(&issue.to_string(), language))
            .collect::<Vec<_>>();
        return Err(WorkspaceError::usage(messages.join("; ")));
    }
    Ok(())
}

fn conflict_message(conflicts: &[MergeConflict], language: &str) -> String {
    let lines = conflicts
        .iter()
        .map(|conflict| {
            fill(
                crate::i18n::text("merge_conflict_line", language),
                &[&conflict.field, &conflict.values.join(", ")],
            )
        })
        .collect::<Vec<_>>();
    fill(
        crate::i18n::text("merge_conflicts", language),
        &[&lines.join("\n")],
    )
}

fn print_preview(
    mode: OutputMode,
    language: &str,
    plan: &MergePlan,
    records: &[Record],
) -> Result<()> {
    let source_ids = plan.source_ids.join(", ");
    let conflicts = plan
        .conflicts
        .iter()
        .map(|conflict| {
            serde_json::json!({
                "field": conflict.field,
                "values": conflict.values,
            })
        })
        .collect::<Vec<_>>();
    let redirected_ids = redirected_record_ids(records, &plan.source_ids, &plan.id);
    if mode.is_json() {
        return print_json(&serde_json::json!({
            "dry_run": true,
            "candidate_id": plan.id,
            "sources": plan.source_ids,
            "ready": plan.is_ready(),
            "conflicts": conflicts,
            "redirected_records": redirected_ids,
            "archived_sources": plan.source_ids.len(),
        }));
    }
    println!(
        "{}",
        fill(
            crate::i18n::text("merge_preview", language),
            &[&plan.id, &source_ids],
        )
    );
    println!(
        "{}",
        fill(
            crate::i18n::text("merge_impact", language),
            &[
                &redirected_ids.len().to_string(),
                &plan.source_ids.len().to_string(),
            ],
        )
    );
    println!(
        "{}",
        fill(
            crate::i18n::text("merge_redirected", language),
            &[&localized_ids(&redirected_ids, language)],
        )
    );
    if !plan.conflicts.is_empty() {
        println!("{}", conflict_message(&plan.conflicts, language));
    }
    Ok(())
}

fn redirected_record_ids(
    records: &[Record],
    source_ids: &[String],
    target_id: &str,
) -> Vec<String> {
    records
        .iter()
        .filter_map(|record| {
            let is_source = record
                .id()
                .is_ok_and(|id| source_ids.iter().any(|source| source == id));
            if is_source {
                return None;
            }
            let parent = record
                .parent()
                .is_some_and(|id| source_ids.iter().any(|source| source == &id));
            let dependency = record
                .depends_on()
                .iter()
                .any(|id| source_ids.iter().any(|source| source == id));
            let body = crate::record::markdown::split_front_matter(&record.raw)
                .map(|(_, body)| {
                    crate::workspace::merge::redirect_body_references(
                        &body,
                        &source_ids
                            .iter()
                            .map(|source| (source.clone(), target_id.to_string()))
                            .collect(),
                    ) != body
                })
                .unwrap_or(false);
            (parent || dependency || body).then(|| record.id().unwrap_or("").to_string())
        })
        .collect()
}

fn localized_ids(ids: &[String], language: &str) -> String {
    if ids.is_empty() {
        crate::output::labels(language).no_matches().to_string()
    } else {
        ids.join(crate::output::labels(language).list_separator())
    }
}
