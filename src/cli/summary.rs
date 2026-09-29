//! `summary`：当前总览与交接摘要。

use crate::error::Result;
use crate::output::labels;
use crate::style::{self, paint};
use crate::view;

use super::Context;
use super::args::SummaryArgs;

/// 生成当前总览，或 `--handoff` 时的交接摘要。
///
/// `--handoff` 影响文本与 JSON 两种呈现：文本走 `view::render_handoff`，JSON 走
/// `view::handoff_json`，不会退化成默认总览。
pub fn summary(context: &Context, args: &SummaryArgs) -> Result<()> {
    let workspace = context.workspace()?;
    let language = workspace.project_language();
    let generated_at = args.at.clone().unwrap_or_else(crate::time::now_iso);
    let overview = view::build(&workspace, generated_at);

    if let Some(target) = args.save.as_deref() {
        let target = if target.is_dir() {
            target.join(format!("{}.md", overview.generated_at.replace(':', "-")))
        } else {
            target.to_path_buf()
        };
        let text = if args.handoff {
            view::render_handoff(&language, &overview)
        } else {
            view::render_text(&language, &overview)
        };
        view::save_snapshot(&target, &text, &overview.generated_at)?;
        if context.mode.is_json() {
            let mut value = if args.handoff {
                view::handoff_json(&overview)
            } else {
                overview.to_json()
            };
            value["saved"] = serde_json::Value::String(target.display().to_string());
            crate::output::print_json(&value)?;
        } else {
            println!(
                "{} {}",
                paint(style::ok(), labels(&language).saved()),
                paint(style::muted(), &target.display().to_string())
            );
        }
        return Ok(());
    }

    if args.handoff {
        if context.mode.is_json() {
            return crate::output::print_json(&view::handoff_json(&overview));
        }
        println!("{}", view::render_handoff(&language, &overview));
        return Ok(());
    }
    view::print_overview(context.mode, &language, &overview)
}
