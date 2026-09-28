//! `add`、`show`、`update`、`log`、`drop`、`restore` 与列表呈现。

use std::collections::{BTreeMap, BTreeSet};

use crate::checks::CheckReport;
use crate::domain::Scalar;
use crate::error::{Result, WorkspaceError};
use crate::kind::{check_field_value, render_template, template_values};
use crate::output::{OutputMode, labels, print_json};
use crate::record::Record;
use crate::time;
use crate::workspace::Config;

use super::Context;
use super::args::{AddArgs, DropArgs, LogArgs, RestoreArgs, UpdateArgs};
use super::declared_groups;

/// 头部规范键序；kind 声明字段插在 `title` 之后，未知键保留在末尾。
const HEAD_KEYS: [&str; 4] = ["id", "kind", "group", "title"];

/// 除 kind 声明字段外，`--set` 还允许写入的规范键。
///
/// `id` 由工具分配，`kind`/`group` 有专用参数，都不走 `--set`。这些键在头部按
/// 「规范键 → kind 声明字段 → 其余规范键」的顺序收尾，模板没有出现且本次未写入时
/// 不主动补空。
const CANONICAL_SET_KEYS: [&str; 4] = [
    "parent",
    "depends_on",
    "completion_note",
    "completion_evidence",
];

pub fn add(context: &Context, args: &AddArgs) -> Result<()> {
    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let id = workspace.transaction(|transaction| {
        let config = transaction.config();
        let kind = config.require_kind(&args.kind)?;
        if !config.has_group(&args.group) {
            return Err(WorkspaceError::usage(format!(
                "unknown group `{}`; declared groups: {}",
                args.group,
                declared_groups(config)
            )));
        }

        let mut values: BTreeMap<String, String> = BTreeMap::new();
        values.insert("group".to_string(), args.group.clone());
        for field in &kind.fields {
            values.insert(field.name.clone(), String::new());
        }
        for (key, value) in &args.set {
            match require_settable(kind, key)? {
                Some(field) => {
                    if !value.is_empty() {
                        check_field_value(kind, field, value)?;
                    }
                }
                // 规范键：校验列表写法，`parent` 只接受单个 ID。
                None => {
                    canonical_scalar(key, value)?;
                }
            }
            values.insert(key.clone(), value.clone());
        }
        let title = args
            .title
            .clone()
            .unwrap_or_else(|| values.get("title").cloned().unwrap_or_default());
        values.insert("title".to_string(), title.clone());

        let id = transaction.allocate_id()?;
        values.insert("id".to_string(), id.clone());
        values.insert("kind".to_string(), kind.name.clone());

        // 模板是头部字段顺序、默认值与正文骨架的来源；显式取值覆盖模板默认值。
        let target = config.items_dir().join(format!("{id}.md"));
        let mut record = match config.template_path(kind) {
            Some(path) => {
                let text = std::fs::read_to_string(&path)
                    .map_err(|error| crate::error::read_error(&path, error))?;
                let rendered = render_template(&text, &template_values(kind, &values));
                // 解析失败要指向即将写入的记录，而不是来源模板。
                crate::record::Record::parse(&target, rendered)?
            }
            None => crate::record::Record::parse(&target, String::new())?,
        };
        record.fields = ordered_fields(&record, kind, &values);

        if let Some(path) = args.body.as_deref() {
            let body = std::fs::read_to_string(path)
                .map_err(|error| crate::error::read_error(path, error))?;
            record.body = crate::record::markdown::BodyDoc::parse(&body);
        }

        let known: Vec<String> = transaction
            .records()
            .iter()
            .filter_map(|existing| existing.id().ok().map(str::to_string))
            .collect();
        check_references(&record, &known)?;

        if writes_completion(kind, &args.set) {
            let missing = crate::checks::missing_completion_parts(config, &record);
            if !missing.is_empty() {
                return Err(WorkspaceError::usage(format!(
                    "setting the completion field to its completion value requires a non-empty {}; add it (or mark the result as unverified) before recording completion",
                    missing.join(" and ")
                )));
            }
        }

        transaction.create(&id, &record.render())?;
        Ok(id)
    })?;

    workspace.refresh()?;
    let record = workspace.index().require(&id)?;
    if !context.mode.is_json() {
        println!("{} {}", labels(&language).created(), id);
    }
    print_record(context.mode, &language, workspace.config(), record)
}

pub fn update(context: &Context, args: &UpdateArgs) -> Result<()> {
    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let expected = workspace.index().require(&args.id)?.raw.clone();
    workspace.transaction(|transaction| {
        let mut record = transaction.current(&args.id)?;
        let config = transaction.config().clone();
        let kind = config.require_kind(record.kind()?)?;

        for (key, value) in &args.set {
            let scalar = match require_settable(kind, key)? {
                Some(field) => {
                    if !value.is_empty() {
                        check_field_value(kind, field, value)?;
                    }
                    Scalar::from_plain(value)
                }
                // 规范键：支持 `parent`/`depends_on` 的列表写法。
                None => canonical_scalar(key, value)?,
            };
            record.set(key, scalar);
        }
        for key in &args.unset {
            require_settable(kind, key)?;
            record.remove(key);
        }
        if let Some(group) = args.group.as_deref() {
            if !config.has_group(group) {
                return Err(WorkspaceError::usage(format!(
                    "unknown group `{group}`; declared groups: {}",
                    declared_groups(&config)
                )));
            }
            record.set("group", Scalar::from_plain(group));
        }
        for (section, body) in &args.section {
            record.body.set_section(section, body);
        }
        for (section, line) in &args.append {
            record.body.append_line(section, line);
        }
        let known: Vec<String> = transaction
            .records()
            .iter()
            .filter_map(|existing| existing.id().ok().map(str::to_string))
            .collect();
        check_references(&record, &known)?;

        if writes_completion(kind, &args.set) {
            let missing = crate::checks::missing_completion_parts(&config, &record);
            if !missing.is_empty() {
                return Err(WorkspaceError::usage(format!(
                    "setting the completion field to its completion value requires a non-empty {}; add one (or mark the result as unverified) before recording completion",
                    missing.join(" and ")
                )));
            }
        }

        let expected = if args.force {
            None
        } else {
            Some(expected.as_str())
        };
        transaction.update(&args.id, expected, &record.render())?;
        Ok(())
    })?;

    workspace.refresh()?;
    let record = workspace.index().require(&args.id)?;
    if !context.mode.is_json() {
        println!("{} {}", labels(&language).updated(), args.id);
    }
    print_record(context.mode, &language, workspace.config(), record)
}

pub fn log(context: &Context, args: &LogArgs) -> Result<()> {
    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let expected = workspace.index().require(&args.id)?.raw.clone();
    let date = match args.date.as_deref() {
        Some(date) => time::parse_date(date).ok_or_else(|| {
            WorkspaceError::usage(format!("expected `--date` as YYYY-MM-DD, got `{date}`"))
        })?,
        None => time::today(),
    };

    workspace.transaction(|transaction| {
        let mut record = transaction.current(&args.id)?;
        record
            .body
            .append_line(&args.section, &format!("- {date}：{}", args.text.trim()));
        transaction.update(&args.id, Some(expected.as_str()), &record.render())?;
        Ok(())
    })?;

    workspace.refresh()?;
    let record = workspace.index().require(&args.id)?;
    if !context.mode.is_json() {
        println!("{} {}", labels(&language).updated(), args.id);
    }
    print_record(context.mode, &language, workspace.config(), record)
}

pub fn drop(context: &Context, args: &DropArgs) -> Result<()> {
    lifecycle(context, &args.id, true, args.reason.as_deref())
}

pub fn restore(context: &Context, args: &RestoreArgs) -> Result<()> {
    lifecycle(context, &args.id, false, None)
}

fn lifecycle(context: &Context, id: &str, dropped: bool, reason: Option<&str>) -> Result<()> {
    let mut workspace = context.locked_workspace()?;
    let language = workspace.project_language(context.language.as_deref());
    let expected = workspace.index().require(id)?.raw.clone();
    workspace.transaction(|transaction| {
        let mut record = transaction.current(id)?;
        record.set("dropped", Scalar::Bool(dropped));
        if let Some(reason) = reason {
            record.body.append_line(
                "进展",
                &format!("- {}：废弃（{}）", time::today(), reason.trim()),
            );
        }
        transaction.update(id, Some(expected.as_str()), &record.render())?;
        Ok(())
    })?;
    workspace.refresh()?;
    let record = workspace.index().require(id)?;
    if !context.mode.is_json() {
        let labels = labels(&language);
        println!(
            "{} {} · {}",
            labels.updated(),
            id,
            crate::output::lifecycle_text(&language, dropped)
        );
    }
    print_record(context.mode, &language, workspace.config(), record)
}

/// 头部字段顺序：规范键 → kind 声明字段 → 其它规范键 → 模板里未识别的字段。
///
/// 显式取值优先；取值为空且模板给了默认值时用模板默认值；两者都没有则写空字符串。
/// 模板未出现的尾部键（parent/depends_on/完成依据）不主动补空，避免污染记录。
fn ordered_fields(
    template: &Record,
    kind: &crate::workspace::KindConfig,
    values: &BTreeMap<String, String>,
) -> Vec<(String, Scalar)> {
    let mut fields: Vec<(String, Scalar)> = Vec::new();
    let push = |key: &str, fields: &mut Vec<(String, Scalar)>| {
        if fields.iter().any(|(existing, _)| existing == key) {
            return;
        }
        let explicit = values.get(key).filter(|value| !value.is_empty());
        let value = match (explicit, template.get(key)) {
            (Some(value), _) => scalar_of(key, value),
            (None, Some(default)) => default.clone(),
            (None, None) => Scalar::Text(String::new()),
        };
        fields.push((key.to_string(), value));
    };
    for key in HEAD_KEYS {
        push(key, &mut fields);
    }
    for field in &kind.fields {
        push(&field.name, &mut fields);
    }
    for key in CANONICAL_SET_KEYS {
        if template.get(key).is_some() || values.get(key).is_some_and(|value| !value.is_empty()) {
            push(key, &mut fields);
        }
    }
    for (key, value) in values {
        if !fields.iter().any(|(existing, _)| existing == key) {
            fields.push((key.clone(), scalar_of(key, value)));
        }
    }
    for (key, value) in &template.fields {
        if !fields.iter().any(|(existing, _)| existing == key) {
            fields.push((key.clone(), value.clone()));
        }
    }
    fields
}

/// 头部字段取值：列表键接受 `[A, B]` 与 `A, B` 两种写法。
///
/// 调用前必须先用 `canonical_scalar` 校验，这里假定 `parent` 至多一个取值。
fn scalar_of(key: &str, value: &str) -> Scalar {
    if value.is_empty() {
        return Scalar::Text(String::new());
    }
    match key {
        "parent" => Scalar::Text(
            parse_reference_list(value)
                .into_iter()
                .next()
                .unwrap_or_default(),
        ),
        "depends_on" => {
            let items = parse_reference_list(value);
            if items.is_empty() {
                Scalar::Text(String::new())
            } else {
                Scalar::List(items.into_iter().map(Scalar::Text).collect())
            }
        }
        _ => Scalar::from_plain(value),
    }
}

/// 规范键的头部取值，并对列表写法做校验（`parent` 只接受单个记录 ID）。
fn canonical_scalar(key: &str, raw: &str) -> Result<Scalar> {
    let items = parse_reference_list(raw);
    match key {
        "parent" => match items.len() {
            0 => Ok(Scalar::Text(String::new())),
            1 => Ok(Scalar::Text(items.into_iter().next().unwrap_or_default())),
            _ => Err(WorkspaceError::usage(format!(
                "`parent` takes a single record ID, got `{raw}`; use `depends_on` for several records"
            ))),
        },
        "depends_on" => {
            if items.is_empty() {
                Ok(Scalar::Text(String::new()))
            } else {
                Ok(Scalar::List(items.into_iter().map(Scalar::Text).collect()))
            }
        }
        _ => Ok(Scalar::from_plain(raw)),
    }
}

/// 解析一列记录 ID：接受 `[A, B]`、`A, B` 与单个 `A`。
fn parse_reference_list(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    let inner = raw
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(raw);
    inner
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

/// 写入时维护引用完整性：`parent`/`depends_on` 必须指向项目内已存在的记录。
fn check_references(record: &Record, known: &[String]) -> Result<()> {
    let own = record.id().unwrap_or("");
    for field in ["parent", "depends_on"] {
        for reference in record.references(field) {
            if reference == own {
                return Err(WorkspaceError::usage(format!(
                    "`{field}` must not reference the record itself (`{own}`)"
                )));
            }
            if !known.iter().any(|id| id == &reference) {
                return Err(WorkspaceError::usage(format!(
                    "`{field}` references unknown record `{reference}`"
                )));
            }
        }
    }
    Ok(())
}

/// 本次显式写入是否把 kind 的完成字段设成了完成值。
fn writes_completion(kind: &crate::workspace::KindConfig, values: &[(String, String)]) -> bool {
    let Some((field, completion_values)) = kind.completion_rule() else {
        return false;
    };
    values.iter().any(|(key, value)| {
        key == field
            && completion_values
                .iter()
                .any(|candidate| candidate == value.trim())
    })
}

/// 呈现一条记录：JSON 或人读文本。
pub fn print_record(
    mode: OutputMode,
    language: &str,
    config: &Config,
    record: &Record,
) -> Result<()> {
    if mode.is_json() {
        print_json(&record_json(config, record))?;
        return Ok(());
    }
    println!("{}", record_text(language, config, record));
    Ok(())
}

/// 呈现一组记录。
pub fn print_records(
    mode: OutputMode,
    language: &str,
    config: &Config,
    records: &[&Record],
) -> Result<()> {
    if mode.is_json() {
        let items: Vec<serde_json::Value> = records
            .iter()
            .map(|record| record_json(config, record))
            .collect();
        print_json(&serde_json::json!({
            "count": items.len(),
            "items": items,
        }))?;
        return Ok(());
    }
    if records.is_empty() {
        println!("{}", labels(language).items());
        return Ok(());
    }
    for record in records {
        println!("{}", record_line(language, config, record));
    }
    Ok(())
}

/// 呈现检查报告；有问题时返回非零退出状态。
pub fn print_report(mode: OutputMode, language: &str, report: &CheckReport) -> Result<()> {
    if mode.is_json() {
        print_json(&report.to_json())?;
    } else {
        let labels = labels(language);
        println!("{}：{}", labels.checked(), report.checked);
        if report.is_ok() {
            println!("{}", labels.ok());
        } else {
            println!("{}：{}", labels.issues(), report.issues.len());
            for issue in &report.issues {
                println!("- {issue}");
            }
        }
    }
    if report.is_ok() {
        Ok(())
    } else {
        Err(WorkspaceError::runtime(format!(
            "worklog check found {} issue(s)",
            report.issues.len()
        )))
    }
}

/// 机器可读的记录形态；键名稳定。
#[must_use]
pub fn record_json(config: &Config, record: &Record) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    if let Ok(kind_name) = record.kind()
        && let Some(kind) = config.kind(kind_name)
    {
        for field in &kind.fields {
            fields.insert(
                field.name.clone(),
                record
                    .get(&field.name)
                    .map_or(serde_json::Value::Null, scalar_json),
            );
        }
    }
    for (key, value) in &record.fields {
        if !fields.contains_key(key) {
            fields.insert(key.clone(), scalar_json(value));
        }
    }
    serde_json::json!({
        "id": record.id().unwrap_or(""),
        "kind": record.kind().unwrap_or(""),
        "group": record.group(),
        "title": record.title(),
        "fields": serde_json::Value::Object(fields),
        "status": record.status(config),
        "parent": record.parent(),
        "depends_on": record.depends_on(),
        "completion_note": record.completion_note(),
        "completion_evidence": record.completion_evidence(),
        "lifecycle": record.lifecycle().as_key(),
        "body": record.body.render(),
        "path": record.path.display().to_string(),
    })
}

fn scalar_json(value: &Scalar) -> serde_json::Value {
    match value {
        Scalar::Text(text) => serde_json::Value::String(text.clone()),
        Scalar::Int(number) => serde_json::Value::from(*number),
        Scalar::Bool(flag) => serde_json::Value::from(*flag),
        Scalar::List(items) => serde_json::Value::Array(items.iter().map(scalar_json).collect()),
        Scalar::Null => serde_json::Value::Null,
    }
}

/// 追加一行人读字段，并记下该键；同一键只打印一次。
fn push_line(
    out: &mut String,
    printed: &mut BTreeSet<String>,
    key: &str,
    label: &str,
    value: &str,
) {
    if !printed.insert(key.to_string()) {
        return;
    }
    out.push_str(&format!("{label}：{value}\n"));
}

/// 一条记录的人读文本。
///
/// 头部四个键用本地化标签，其余头部字段按原始键打印；每个键只出现一次。
#[must_use]
pub fn record_text(language: &str, config: &Config, record: &Record) -> String {
    let labels = labels(language);
    let mut out = String::new();
    let mut printed: BTreeSet<String> = BTreeSet::new();
    // 生命周期单独成行，`dropped` 头部键不再重复打印。
    printed.insert("dropped".to_string());

    push_line(
        &mut out,
        &mut printed,
        "id",
        labels.id(),
        record.id().unwrap_or(""),
    );
    push_line(
        &mut out,
        &mut printed,
        "kind",
        labels.kind(),
        record.kind().unwrap_or(""),
    );
    push_line(
        &mut out,
        &mut printed,
        "group",
        labels.group(),
        record.group(),
    );
    push_line(
        &mut out,
        &mut printed,
        "title",
        labels.title(),
        record.title(),
    );
    if let Some(status) = record.status(config) {
        let key = record
            .kind()
            .ok()
            .and_then(|name| config.kind(name))
            .and_then(crate::workspace::KindConfig::status_field)
            .unwrap_or("status");
        push_line(&mut out, &mut printed, key, labels.status(), &status);
    }
    if let Some(parent) = record.parent() {
        push_line(&mut out, &mut printed, "parent", labels.parent(), &parent);
    }
    let depends_on = record.depends_on();
    if !depends_on.is_empty() {
        push_line(
            &mut out,
            &mut printed,
            "depends_on",
            labels.depends_on(),
            &depends_on.join("、"),
        );
    }
    push_line(
        &mut out,
        &mut printed,
        "lifecycle",
        labels.lifecycle(),
        crate::output::lifecycle_text(language, record.lifecycle().is_dropped()),
    );

    if let Ok(kind_name) = record.kind()
        && let Some(kind) = config.kind(kind_name)
    {
        for field in &kind.fields {
            let value = record
                .get(&field.name)
                .map_or_else(String::new, Scalar::display);
            push_line(&mut out, &mut printed, &field.name, &field.name, &value);
        }
    }
    let note = record.completion_note();
    if !note.trim().is_empty() {
        push_line(
            &mut out,
            &mut printed,
            "completion_note",
            labels.completion_note(),
            note.trim(),
        );
    }
    let evidence = record.completion_evidence();
    if !evidence.trim().is_empty() {
        push_line(
            &mut out,
            &mut printed,
            "completion_evidence",
            labels.completion_evidence(),
            evidence.trim(),
        );
    }
    // 头部里剩下、前面没有专门打印的字段按原始键输出。
    for (key, value) in &record.fields {
        push_line(&mut out, &mut printed, key, key, &Scalar::display(value));
    }

    out.push_str(&format!("\n{}：\n{}", labels.body(), record.body.render()));
    out
}

/// 列表中的一行。
#[must_use]
pub fn record_line(language: &str, config: &Config, record: &Record) -> String {
    let kind = record.kind().unwrap_or("");
    let status = record
        .status(config)
        .map_or_else(String::new, |status| format!(" · {status}"));
    let lifecycle = if record.lifecycle().is_dropped() {
        format!(" · {}", labels(language).dropped())
    } else {
        String::new()
    };
    format!(
        "{} {} · {kind} · {}{status}{lifecycle}",
        record.id().unwrap_or(""),
        record.title(),
        record.group()
    )
}

fn declared_fields(kind: &crate::workspace::KindConfig) -> String {
    if kind.fields.is_empty() {
        "none declared".to_string()
    } else {
        kind.fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// 校验 `--set`/`--unset` 的键。
///
/// 返回 kind 声明的字段（需要按类型校验取值）；规范键返回 `None`（原样写入头部）；
/// 其余键属用法错误。
fn require_settable<'a>(
    kind: &'a crate::workspace::config::KindConfig,
    key: &str,
) -> Result<Option<&'a crate::workspace::config::FieldDef>> {
    if let Some(field) = kind.field(key) {
        return Ok(Some(field));
    }
    if CANONICAL_SET_KEYS.contains(&key) {
        return Ok(None);
    }
    Err(WorkspaceError::usage(format!(
        "kind `{}` declares no field `{key}`; declared fields: {}; also settable: {}",
        kind.name,
        declared_fields(kind),
        CANONICAL_SET_KEYS.join(", ")
    )))
}
