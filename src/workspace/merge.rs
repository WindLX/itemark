//! 合并记录的预检与候选记录构造。
//!
//! I/O、锁和归档由 CLI/事务负责；这里仅根据当前索引和用户显式选择构造候选，
//! 让 dry-run 与真正写入共用同一套冲突判定。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::PathBuf;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::domain::{Field, Scalar};
use crate::error::{Result, WorkspaceError};
use crate::record::markdown::Section;
use crate::record::{BodyDoc, Record};
use crate::workspace::config::Config;

#[derive(Debug, Clone, Default)]
pub struct MergeOptions {
    pub source_ids: Vec<String>,
    pub title: String,
    pub group: String,
    /// 只允许 kind 声明字段；值为空时显式清空。
    pub set: Vec<(String, String)>,
    /// None 表示未指定；Some(None) 显式清空父项。
    pub parent: Option<Option<String>>,
    pub completion_note: Option<String>,
    pub completion_evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeConflict {
    pub field: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MergePlan {
    pub id: String,
    pub source_ids: Vec<String>,
    pub record: Record,
    pub conflicts: Vec<MergeConflict>,
}

#[derive(Debug, Clone, Default)]
pub struct MergeOutcome {
    pub id: String,
    pub redirected_records: Vec<String>,
    pub archived_sources: Vec<String>,
}

impl MergePlan {
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.conflicts.is_empty()
    }
}

/// 从已锁定事务提供的当前记录构造合并预览。候选 ID 仅供展示，执行前必须重建计划。
pub fn prepare(
    config: &Config,
    all_records: &[Record],
    candidate_id: &str,
    options: &MergeOptions,
) -> Result<MergePlan> {
    let source_ids = distinct_ids(&options.source_ids);
    if source_ids.len() < 2 {
        return Err(WorkspaceError::usage(
            "merge requires at least two distinct source IDs",
        ));
    }
    if options.title.trim().is_empty() {
        return Err(WorkspaceError::usage("merge title must not be empty"));
    }
    if !config.has_group(&options.group) {
        return Err(WorkspaceError::usage(format!(
            "unknown group `{}`",
            options.group
        )));
    }

    let mut source_records = Vec::with_capacity(source_ids.len());
    for id in &source_ids {
        let record = all_records
            .iter()
            .find(|record| record.id().is_ok_and(|record_id| record_id == id))
            .ok_or_else(|| WorkspaceError::usage(format!("unknown Itemark item `{id}`")))?;
        if record.lifecycle().is_dropped() {
            return Err(WorkspaceError::usage(format!(
                "source `{id}` is dropped; restore it before merging"
            )));
        }
        source_records.push(record);
    }

    let kind_name = source_records[0].kind()?.to_string();
    if let Some(other) = source_records
        .iter()
        .find(|record| record.kind().is_ok_and(|kind| kind != kind_name))
    {
        return Err(WorkspaceError::usage(format!(
            "merge sources must have the same kind; `{}` has kind `{}`",
            other.id()?,
            other.kind()?
        )));
    }
    let kind = config.require_kind(&kind_name)?;

    let mut explicit = BTreeMap::new();
    for (field_name, value) in &options.set {
        let field = kind.field(field_name).ok_or_else(|| {
            WorkspaceError::usage(format!(
                "`--set` field `{field_name}` is not declared by kind `{kind_name}`"
            ))
        })?;
        if field_name == "title" {
            return Err(WorkspaceError::usage(
                "merge title must be supplied with `--title`, not `--set title`",
            ));
        }
        if !value.is_empty() {
            crate::kind::check_field_value(kind, field, value)?;
        }
        if explicit
            .insert(field_name.clone(), Scalar::from_plain(value))
            .is_some()
        {
            return Err(WorkspaceError::usage(format!(
                "merge field `{field_name}` was set more than once"
            )));
        }
    }

    let source_set: BTreeSet<&str> = source_ids.iter().map(String::as_str).collect();
    let mut fields: Vec<Field> = vec![
        ("id".to_string(), Scalar::Text(candidate_id.to_string())),
        ("kind".to_string(), Scalar::Text(kind_name.clone())),
        ("group".to_string(), Scalar::Text(options.group.clone())),
        ("title".to_string(), Scalar::Text(options.title.clone())),
    ];
    let mut conflicts = Vec::new();

    // Kind 字段按定义顺序保留，并允许用 --set 显式解决冲突。
    for field in &kind.fields {
        let value = if field.name == "title" {
            Scalar::Text(options.title.clone())
        } else if let Some(value) = explicit.remove(&field.name) {
            value
        } else {
            merged_scalar(&source_records, &field.name, &mut conflicts)
        };
        if field.required
            && value.is_empty()
            && !conflicts
                .iter()
                .any(|conflict| conflict.field == field.name)
        {
            conflicts.push(MergeConflict {
                field: field.name.clone(),
                values: vec!["required field is empty".to_string()],
            });
        }
        fields.push((field.name.clone(), value));
    }

    // 保留 kind 未声明但记录中已有的单值元数据；冲突字段无法通过 --set 解决。
    let mut extra_names = BTreeSet::new();
    for record in &source_records {
        for (name, _) in &record.fields {
            if !is_reserved(name) && kind.field(name).is_none() {
                extra_names.insert(name.clone());
            }
        }
    }
    for name in extra_names {
        fields.push((
            name.clone(),
            merged_scalar(&source_records, &name, &mut conflicts),
        ));
    }

    let parent = resolve_parent(
        &source_records,
        options.parent.as_ref(),
        &source_set,
        &mut conflicts,
    )?;
    if let Some(parent) = parent {
        if source_set.contains(parent.as_str()) {
            return Err(WorkspaceError::usage(format!(
                "merge parent `{parent}` cannot be one of the source records"
            )));
        }
        if !all_records
            .iter()
            .any(|record| record.id().is_ok_and(|id| id == parent))
        {
            return Err(WorkspaceError::usage(format!(
                "merge parent `{parent}` does not exist"
            )));
        }
        fields.push(("parent".to_string(), Scalar::Text(parent)));
    }

    let mut dependencies = Vec::new();
    for record in &source_records {
        for dependency in record.depends_on() {
            if !source_set.contains(dependency.as_str()) && !dependencies.contains(&dependency) {
                dependencies.push(dependency);
            }
        }
    }
    if !dependencies.is_empty() {
        fields.push((
            "depends_on".to_string(),
            Scalar::List(dependencies.into_iter().map(Scalar::Text).collect()),
        ));
    }

    for (field, explicit_value) in [
        ("completion_note", options.completion_note.as_deref()),
        (
            "completion_evidence",
            options.completion_evidence.as_deref(),
        ),
    ] {
        let value = match explicit_value {
            Some(value) => Scalar::from_plain(value),
            None => merged_scalar(&source_records, field, &mut conflicts),
        };
        if !value.is_empty() {
            fields.push((field.to_string(), value));
        }
    }

    fields.push((
        "merged_from".to_string(),
        Scalar::List(source_ids.iter().cloned().map(Scalar::Text).collect()),
    ));
    fields.push(("needs_review".to_string(), Scalar::Bool(true)));

    let body = merged_body(&source_records, &kind.required_sections);
    let raw = crate::record::NewRecord { fields, body }.render();
    let record = Record::parse(&PathBuf::from(format!("{candidate_id}.md")), raw)?;

    Ok(MergePlan {
        id: candidate_id.to_string(),
        source_ids,
        record,
        conflicts,
    })
}

/// 按预检计划写入新记录、重定向现有关系并归档来源。
///
/// 调用方须在持有项目锁时重新构造计划并通过 kind/check 校验。每次更新仍会比较
/// 原文；遇到外部编辑时停止并说明此前已完成的步骤。
pub fn apply(
    transaction: &mut crate::workspace::Transaction<'_>,
    plan: &MergePlan,
    records: &[Record],
) -> Result<MergeOutcome> {
    if !plan.is_ready() {
        return Err(WorkspaceError::usage(
            "merge has unresolved field conflicts",
        ));
    }
    let source_set: BTreeSet<&str> = plan.source_ids.iter().map(String::as_str).collect();
    let redirects: BTreeMap<String, String> = plan
        .source_ids
        .iter()
        .map(|id| (id.clone(), plan.id.clone()))
        .collect();

    let mut updates = Vec::<(String, String, String)>::new();
    for record in records {
        let id = record.id()?.to_string();
        if source_set.contains(id.as_str()) {
            continue;
        }
        let mut updated = record.clone();
        let mut changed = false;

        if let Some(parent) = updated.parent()
            && source_set.contains(parent.as_str())
        {
            updated.set("parent", Scalar::Text(plan.id.clone()));
            changed = true;
        }

        let mut dependencies = Vec::new();
        for dependency in updated.depends_on() {
            let replacement = redirects.get(&dependency).unwrap_or(&dependency);
            if !dependencies.contains(replacement) {
                dependencies.push(replacement.clone());
            }
        }
        let previous_dependencies = updated.depends_on();
        if dependencies != previous_dependencies {
            if dependencies.is_empty() {
                updated.remove("depends_on");
            } else {
                updated.set(
                    "depends_on",
                    Scalar::List(dependencies.into_iter().map(Scalar::Text).collect()),
                );
            }
            changed = true;
        }

        let original_body = raw_body(&record.raw)?;
        let redirected_body = redirect_body_references(original_body, &redirects);
        if redirected_body != original_body {
            changed = true;
        }
        if changed {
            let text = render_with_body(&updated, &redirected_body);
            updates.push((id, record.raw.clone(), text));
        }
    }

    // 在首次写入前确认计划中所有受影响原文仍与锁内快照一致。
    for (id, expected, _) in &updates {
        let current = transaction.current(id)?;
        if current.raw != *expected {
            return Err(WorkspaceError::runtime(format!(
                "Itemark item `{id}` changed outside this operation; re-read it and retry"
            )));
        }
    }
    for id in &plan.source_ids {
        let current = transaction.current(id)?;
        let expected = records
            .iter()
            .find(|record| record.id().ok() == Some(id.as_str()))
            .map(|record| record.raw.as_str())
            .ok_or_else(|| WorkspaceError::usage(format!("unknown Itemark item `{id}")))?;
        if current.raw != expected {
            return Err(WorkspaceError::runtime(format!(
                "Itemark item `{id}` changed outside this operation; re-read it and retry"
            )));
        }
    }

    transaction.create(&plan.id, &plan.record.render())?;
    let mut outcome = MergeOutcome {
        id: plan.id.clone(),
        ..MergeOutcome::default()
    };
    for (id, expected, text) in updates {
        if let Err(error) = transaction.update(&id, Some(&expected), &text) {
            return Err(WorkspaceError::runtime(format!(
                "merge `{}` created; reference updates stopped at `{id}` after {} record(s): {error}",
                plan.id,
                outcome.redirected_records.len()
            )));
        }
        outcome.redirected_records.push(id);
    }

    for id in &plan.source_ids {
        let current = match transaction.current(id) {
            Ok(record) => record,
            Err(error) => {
                return Err(WorkspaceError::runtime(format!(
                    "merge `{}` created and {} record(s) redirected; could not update source `{id}`: {error}",
                    plan.id,
                    outcome.redirected_records.len()
                )));
            }
        };
        let mut source = current.clone();
        let mut merged_into = source.references("merged_into");
        if !merged_into.contains(&plan.id) {
            merged_into.push(plan.id.clone());
        }
        let merged_into = if merged_into.len() == 1 {
            Scalar::Text(merged_into.remove(0))
        } else {
            Scalar::List(merged_into.into_iter().map(Scalar::Text).collect())
        };
        source.set("merged_into", merged_into);
        let body = raw_body(&current.raw)?;
        let text = render_with_body(&source, body);
        if let Err(error) = transaction.update(id, Some(&current.raw), &text) {
            return Err(WorkspaceError::runtime(format!(
                "merge `{}` created and {} record(s) redirected; source metadata update stopped at `{id}`: {error}",
                plan.id,
                outcome.redirected_records.len()
            )));
        }
    }

    match transaction.archive_many(&plan.source_ids) {
        Ok(_) => outcome.archived_sources = plan.source_ids.clone(),
        Err(error) => {
            return Err(WorkspaceError::runtime(format!(
                "merge `{}` created; {} record(s) redirected and source metadata updated, but source archival failed: {error}",
                plan.id,
                outcome.redirected_records.len()
            )));
        }
    }
    Ok(outcome)
}

fn distinct_ids(ids: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for id in ids {
        if !out.contains(id) {
            out.push(id.clone());
        }
    }
    out
}

fn is_reserved(field: &str) -> bool {
    matches!(
        field,
        "id" | "kind"
            | "group"
            | "title"
            | "parent"
            | "depends_on"
            | "dropped"
            | "merged_from"
            | "merged_into"
            | "needs_review"
            | "completion_note"
            | "completion_evidence"
    )
}

fn merged_scalar(sources: &[&Record], field: &str, conflicts: &mut Vec<MergeConflict>) -> Scalar {
    let mut values = Vec::<Scalar>::new();
    for source in sources {
        if let Some(value) = source.get(field).filter(|value| !value.is_empty())
            && !values.contains(value)
        {
            values.push(value.clone());
        }
    }
    if values.len() > 1 {
        let mut origins = Vec::new();
        for source in sources {
            if let Some(value) = source.get(field).filter(|value| !value.is_empty()) {
                origins.push(format!(
                    "{}={}",
                    source.id().unwrap_or("?"),
                    value.display()
                ));
            }
        }
        conflicts.push(MergeConflict {
            field: field.to_string(),
            values: origins,
        });
        Scalar::Null
    } else {
        values.pop().unwrap_or(Scalar::Null)
    }
}

fn resolve_parent(
    sources: &[&Record],
    explicit: Option<&Option<String>>,
    source_ids: &BTreeSet<&str>,
    conflicts: &mut Vec<MergeConflict>,
) -> Result<Option<String>> {
    if let Some(explicit) = explicit {
        return Ok(explicit
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .map(str::to_string));
    }
    let mut parents = Vec::<(String, String)>::new();
    for source in sources {
        if let Some(parent) = source.parent()
            && !source_ids.contains(parent.as_str())
            && !parents.iter().any(|(_, value)| value == &parent)
        {
            parents.push((source.id().unwrap_or("?").to_string(), parent));
        }
    }
    if parents.len() > 1 {
        conflicts.push(MergeConflict {
            field: "parent".to_string(),
            values: parents
                .into_iter()
                .map(|(source, parent)| format!("{source}={parent}"))
                .collect(),
        });
        Ok(None)
    } else {
        Ok(parents.pop().map(|(_, parent)| parent))
    }
}

fn merged_body(sources: &[&Record], required_sections: &[String]) -> BodyDoc {
    let mut titles = Vec::<String>::new();
    for title in required_sections {
        if !titles.contains(title) {
            titles.push(title.clone());
        }
    }
    for source in sources {
        for title in source.body.titles() {
            if !titles.contains(&title) {
                titles.push(title);
            }
        }
    }

    let mut sections = Vec::new();
    for title in titles {
        let mut content = String::new();
        for source in sources {
            if let Ok(id) = source.id() {
                let text = source
                    .body
                    .section(&title)
                    .map_or("", |section| section.body.as_str());
                if text.trim().is_empty() {
                    continue;
                }
                if !content.is_empty() {
                    content.push_str("\n\n");
                }
                content.push_str("### ");
                content.push_str(id);
                content.push_str(" — ");
                content.push_str(source.title());
                content.push_str("\n\n");
                content.push_str(text.trim());
            }
        }
        sections.push(Section {
            title,
            body: content,
        });
    }

    let preamble = sources
        .iter()
        .filter_map(|source| {
            let id = source.id().ok()?;
            let text = source.body.preamble.trim();
            (!text.is_empty()).then(|| format!("### {id} — {}\n\n{text}", source.title()))
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    if !preamble.is_empty() {
        sections.push(Section {
            title: "合并来源前言".to_string(),
            body: preamble,
        });
    }
    BodyDoc {
        preamble: String::new(),
        sections,
    }
}

/// 把普通正文中的来源引用改到合并记录，保留别名、代码示例和历史进展原文。
pub fn redirect_body_references(markdown: &str, redirects: &BTreeMap<String, String>) -> String {
    let excluded = excluded_ranges(markdown);
    let mut replacements = Vec::<(Range<usize>, String)>::new();
    let mut cursor = 0;
    let mut range_index = 0;
    while cursor < markdown.len() {
        let Some(relative) = markdown[cursor..].find("[[") else {
            break;
        };
        let start = cursor + relative;
        while range_index < excluded.len() && excluded[range_index].end <= start {
            range_index += 1;
        }
        if let Some(range) = excluded.get(range_index)
            && range.start <= start
            && start < range.end
        {
            cursor = range.end;
            continue;
        }
        let line_end = markdown[start..]
            .find('\n')
            .map_or(markdown.len(), |offset| start + offset);
        let Some(close_offset) = markdown[start + 2..line_end].find("]]") else {
            cursor = start + 2;
            continue;
        };
        let content_start = start + 2;
        let content_end = content_start + close_offset;
        let content = &markdown[content_start..content_end];
        let id = content.split_once('|').map_or(content, |(id, _)| id);
        if let Some(new_id) = redirects.get(id) {
            replacements.push((content_start..content_start + id.len(), new_id.clone()));
        }
        cursor = content_end + 2;
    }

    if replacements.is_empty() {
        return markdown.to_string();
    }
    let mut out = String::with_capacity(markdown.len());
    let mut previous = 0;
    for (range, replacement) in replacements {
        out.push_str(&markdown[previous..range.start]);
        out.push_str(&replacement);
        previous = range.end;
    }
    out.push_str(&markdown[previous..]);
    out
}

fn excluded_ranges(markdown: &str) -> Vec<Range<usize>> {
    let mut code = Vec::<Range<usize>>::new();
    let mut histories = Vec::<(String, Range<usize>)>::new();
    let mut code_start = None;
    let mut heading_start = None;
    for (event, range) in Parser::new(markdown).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => code_start = Some(range.start),
            Event::End(TagEnd::CodeBlock) => {
                if let Some(start) = code_start.take() {
                    code.push(start..range.end);
                }
            }
            Event::Code(_) => code.push(range),
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => heading_start = Some(range.start),
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                if let Some(start) = heading_start.take() {
                    let title = markdown[start..range.end]
                        .trim_start_matches('#')
                        .trim_end_matches('#')
                        .trim()
                        .to_lowercase();
                    histories.push((title, start..range.end));
                }
            }
            _ => {}
        }
    }

    let mut excluded = code;
    for index in 0..histories.len() {
        let (title, heading) = &histories[index];
        if matches!(
            title.as_str(),
            "历史进展" | "进展" | "history" | "progress" | "historical progress"
        ) {
            let end = histories
                .get(index + 1)
                .map_or(markdown.len(), |(_, next)| next.start);
            excluded.push(heading.start..end);
        }
    }
    excluded.sort_by_key(|range| range.start);
    excluded
}

fn raw_body(raw: &str) -> Result<&str> {
    let mut offset = 0;
    let mut lines = raw.split_inclusive('\n');
    let first = lines
        .next()
        .ok_or_else(|| WorkspaceError::runtime("record has no front matter"))?;
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return Err(WorkspaceError::runtime("record has no front matter"));
    }
    offset += first.len();
    for line in lines {
        let marker = line.trim_end_matches(['\r', '\n']);
        let line_start = offset;
        offset += line.len();
        if marker == "---" || marker == "..." {
            return Ok(&raw[offset..]);
        }
        if line_start > raw.len() {
            break;
        }
    }
    Err(WorkspaceError::runtime("record front matter is not closed"))
}

fn render_with_body(record: &Record, body: &str) -> String {
    let front = crate::domain::front_matter::render(&record.fields);
    format!("---\n{}\n---\n{body}", front.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::config::{FieldDef, KindConfig};

    fn config() -> Config {
        Config {
            project_dir: PathBuf::from("."),
            config_path: PathBuf::from("itemark.toml"),
            language: "zh-CN".into(),
            root: PathBuf::from("itemark"),
            kinds: vec![KindConfig {
                name: "work".into(),
                description: String::new(),
                template: None,
                required_sections: vec!["目标".into(), "历史进展".into()],
                completion_field: None,
                completion_values: Vec::new(),
                fields: vec![FieldDef {
                    name: "phase".into(),
                    field_type: "enum".into(),
                    required: true,
                    values: vec!["doing".into(), "done".into()],
                }],
            }],
            groups: vec!["a".into(), "b".into(), "c".into()],
            raw_text: String::new(),
        }
    }

    fn record(id: &str, group: &str, phase: &str, deps: &str, body: &str) -> Record {
        Record::parse(
            &PathBuf::from(format!("{id}.md")),
            format!(
                "---\nid: {id}\nkind: work\ngroup: {group}\ntitle: {id}\nphase: {phase}\ndepends_on: [{deps}]\n---\n{body}"
            ),
        )
        .unwrap()
    }

    fn options(ids: &[&str]) -> MergeOptions {
        MergeOptions {
            source_ids: ids.iter().map(|id| (*id).to_string()).collect(),
            title: "合并项".into(),
            group: "c".into(),
            ..MergeOptions::default()
        }
    }

    #[test]
    fn prepare_deduplicates_sources_and_external_dependencies() {
        let sources = vec![
            record(
                "IM-1",
                "a",
                "doing",
                "IM-9, IM-2",
                "## 目标\n\n甲目标\n\n## 历史进展\n\n甲历史",
            ),
            record(
                "IM-2",
                "b",
                "doing",
                "IM-9, IM-1",
                "## 目标\n\n乙目标\n\n## 历史进展\n\n乙历史",
            ),
        ];
        let plan = prepare(
            &config(),
            &sources,
            "IM-10",
            &options(&["IM-1", "IM-2", "IM-1"]),
        )
        .unwrap();
        assert!(plan.is_ready());
        assert_eq!(plan.source_ids, ["IM-1", "IM-2"]);
        assert_eq!(plan.record.references("depends_on"), ["IM-9"]);
        assert_eq!(plan.record.references("merged_from"), ["IM-1", "IM-2"]);
        assert!(
            plan.record
                .body
                .section("目标")
                .unwrap()
                .body
                .contains("### IM-1")
        );
        assert!(
            plan.record
                .body
                .section("历史进展")
                .unwrap()
                .body
                .contains("乙历史")
        );
    }

    #[test]
    fn conflicting_kind_field_requires_explicit_value() {
        let sources = vec![
            record("IM-1", "a", "doing", "", "## 目标\n\n甲"),
            record("IM-2", "b", "done", "", "## 目标\n\n乙"),
        ];
        let plan = prepare(&config(), &sources, "IM-3", &options(&["IM-1", "IM-2"])).unwrap();
        assert!(!plan.is_ready());
        assert_eq!(plan.conflicts[0].field, "phase");

        let mut resolved = options(&["IM-1", "IM-2"]);
        resolved.set.push(("phase".into(), "doing".into()));
        let plan = prepare(&config(), &sources, "IM-3", &resolved).unwrap();
        assert!(plan.is_ready());
        assert_eq!(plan.record.get("phase").unwrap().display(), "doing");
    }

    #[test]
    fn redirects_aliases_but_keeps_code_and_history() {
        let mut redirects = BTreeMap::new();
        redirects.insert("IM-1".to_string(), "IM-8".to_string());
        let source = "正文 [[IM-1]] 与 [[IM-1|标签]]，`[[IM-1]]`\n\n```md\n[[IM-1]]\n```\n\n## 历史进展\n\n旧 [[IM-1]]\n\n## 进展\n\n更早的日志 [[IM-1]]\n\n## 目标\n\n新 [[IM-1]]";
        let actual = redirect_body_references(source, &redirects);
        assert_eq!(
            actual,
            "正文 [[IM-8]] 与 [[IM-8|标签]]，`[[IM-1]]`\n\n```md\n[[IM-1]]\n```\n\n## 历史进展\n\n旧 [[IM-1]]\n\n## 进展\n\n更早的日志 [[IM-1]]\n\n## 目标\n\n新 [[IM-8]]"
        );
    }
}
