//! 显式移动记录到归档目录或移回活动记录目录。

use super::Context;
use super::args::{ArchiveArgs, UnarchiveArgs};
use crate::error::Result;
use crate::output::{labels, print_json};

pub fn archive(context: &Context, args: &ArchiveArgs) -> Result<()> {
    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language();
    let ids = unique_ids(&args.ids);
    workspace.transaction(|transaction| transaction.archive_many(&ids))?;
    report(
        context,
        &language,
        "archived",
        labels(&language).archive_completed(),
        &ids,
    )
}

pub fn unarchive(context: &Context, args: &UnarchiveArgs) -> Result<()> {
    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language();
    let ids = unique_ids(&args.ids);
    workspace.transaction(|transaction| transaction.unarchive_many(&ids))?;
    report(
        context,
        &language,
        "unarchived",
        labels(&language).unarchive_completed(),
        &ids,
    )
}

fn report(
    context: &Context,
    language: &str,
    key: &str,
    template: &str,
    ids: &[String],
) -> Result<()> {
    if context.mode.is_json() {
        let value = if key == "archived" {
            serde_json::json!({ "archived": ids, "count": ids.len() })
        } else {
            serde_json::json!({ "unarchived": ids, "count": ids.len() })
        };
        return print_json(&value);
    }
    let ids = ids.join(labels(language).list_separator());
    println!("{}", crate::output::fill(template, &[&ids]));
    Ok(())
}

fn unique_ids(ids: &[String]) -> Vec<String> {
    let mut unique = Vec::new();
    for id in ids {
        if !unique.contains(id) {
            unique.push(id.clone());
        }
    }
    unique
}
