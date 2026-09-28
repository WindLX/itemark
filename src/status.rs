//! 记录业务状态与完成依据的唯一判定。
//!
//! 一条记录「当前算什么状态」以及「完成依据是否齐备」都只在这里决定：
//! `show`、`list`、`summary` 与 `check` 都经由本模块，不再各自解释 kind 声明。
//! 判定只依赖配置与记录本身，不做 I/O。

use crate::domain::section;
use crate::record::Record;
use crate::workspace::{Config, KindConfig};

/// 记录在业务上的位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Todo,
    InProgress,
    Blocked,
    /// 已达完成值，但缺少完成说明或证据（或按显式未验证声明标注）。
    Unverified,
    Done,
    /// 没有可判定的业务状态：kind 未声明状态字段，或字段没有可识别的取值。
    NoStatus,
}

impl State {
    #[must_use]
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Unverified => "done_unverified",
            Self::Done => "done",
            Self::NoStatus => "none",
        }
    }

    #[must_use]
    pub fn all() -> [Self; 6] {
        [
            Self::InProgress,
            Self::Blocked,
            Self::Unverified,
            Self::Todo,
            Self::Done,
            Self::NoStatus,
        ]
    }
}

/// 一条记录的业务状态。
///
/// 状态字段由 `KindConfig::status_field` 决定；已声明完成字段与完成值的 kind，
/// 其完成值只有在完成依据齐备时才算 `Done`，否则是 `Unverified`。
#[must_use]
pub fn of(config: &Config, record: &Record) -> State {
    let Some(kind) = record.kind().ok().and_then(|name| config.kind(name)) else {
        return State::NoStatus;
    };
    let Some(field) = kind.status_field() else {
        return State::NoStatus;
    };
    let Some(value) = record.get(field) else {
        return State::NoStatus;
    };
    let value = value.display();
    let value = value.trim();
    if value.is_empty() {
        return State::NoStatus;
    }

    if applies_at_completion(kind, field, value) {
        // 显式标注未验证的完成说明只是「不需要再补证据」，并不等于已验证。
        let declared_unverified = is_unverified_note(&record.completion_note());
        return if declared_unverified || !completion_gaps(config, record).is_empty() {
            State::Unverified
        } else {
            State::Done
        };
    }

    match value {
        "todo" => State::Todo,
        "in_progress" => State::InProgress,
        "blocked" => State::Blocked,
        // kind 没有声明完成字段与完成值，就不套用完成规则：完成值即完成。
        "done" => State::Done,
        // 既不是工作状态，也不是声明过的完成值：不替使用者臆断状态。
        _ => State::NoStatus,
    }
}

/// 记录当前取值是否就是该 kind 声明的完成值。
fn applies_at_completion(kind: &KindConfig, field: &str, value: &str) -> bool {
    kind.completion_rule().is_some_and(|(rule_field, values)| {
        rule_field == field && values.iter().any(|v| v == value)
    })
}

/// 完成依据的缺口：返回仍需补齐的完成说明或证据名称。
///
/// 这是完成规则的唯一实现，写入时与 `check` 复核时都调用它。kind 没有声明完成字段
/// 与完成值、或记录尚未达到完成值时返回空。
#[must_use]
pub fn completion_gaps(config: &Config, record: &Record) -> Vec<String> {
    let Ok(kind_name) = record.kind() else {
        return Vec::new();
    };
    let Some(kind) = config.kind(kind_name) else {
        return Vec::new();
    };
    let Some((field, values)) = kind.completion_rule() else {
        return Vec::new();
    };
    let current = record
        .get(field)
        .map_or_else(String::new, crate::domain::Scalar::display);
    if !values.iter().any(|value| value == current.trim()) {
        return Vec::new();
    }

    let note = record.completion_note();
    if is_unverified_note(&note) {
        return Vec::new();
    }
    let mut missing = Vec::new();
    if note.trim().is_empty() {
        missing.push(section::COMPLETION.to_string());
    }
    if record.completion_evidence().trim().is_empty() {
        missing.push(section::EVIDENCE.to_string());
    }
    missing
}

/// 完成说明以显式标注开头时视为「已声明未验证」，不再要求证据。
///
/// 只有形如 `未验证：说明` / `unverified: note` 的显式标注才算声明；正文里偶然出现
/// 「未验证」字样的普通说明仍然按缺少完成依据处理。
#[must_use]
pub fn is_unverified_note(note: &str) -> bool {
    let note = note.trim().to_lowercase();
    for marker in ["unverified", "未验证"] {
        if let Some(rest) = note.strip_prefix(marker) {
            let rest = rest.trim_start();
            return rest.is_empty() || rest.starts_with(':') || rest.starts_with('：');
        }
    }
    false
}

/// 本次显式写入是否把 kind 的完成字段设成了完成值。
#[must_use]
pub fn writes_completion(kind: &KindConfig, values: &[(String, String)]) -> bool {
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
