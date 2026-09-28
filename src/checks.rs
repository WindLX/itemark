//! 记录检查：必填字段、基础类型、枚举取值、必填分节、关系目标与完成依据。
//!
//! `check` 对全项目应用同一套规则；普通写入只维护语法、ID 与引用完整性，不运行整份
//! 检查。

use std::collections::BTreeMap;
use std::fmt;

use crate::kind::check_field_value;
use crate::record::Record;
use crate::workspace::Workspace;
use crate::workspace::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueKind {
    MissingField,
    FieldValue,
    MissingSection,
    DuplicateId,
    UnknownKind,
    UnknownGroup,
    BrokenReference,
    CompletionEvidence,
}

impl IssueKind {
    #[must_use]
    pub fn as_key(self) -> &'static str {
        match self {
            Self::MissingField => "missing_field",
            Self::FieldValue => "field_value",
            Self::MissingSection => "missing_section",
            Self::DuplicateId => "duplicate_id",
            Self::UnknownKind => "unknown_kind",
            Self::UnknownGroup => "unknown_group",
            Self::BrokenReference => "broken_reference",
            Self::CompletionEvidence => "completion_evidence",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub kind: IssueKind,
    pub target: String,
    pub detail: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.target, self.detail)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckReport {
    pub issues: Vec<Issue>,
    pub checked: usize,
}

impl CheckReport {
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    #[must_use]
    pub fn count_of(&self, kind: IssueKind) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.kind == kind)
            .count()
    }

    /// 某个目标记录的问题。
    #[must_use]
    pub fn issues_for(&self, target: &str) -> Vec<&Issue> {
        self.issues
            .iter()
            .filter(|issue| issue.target == target)
            .collect()
    }

    /// 机器可读形态；键名稳定。
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "checked": self.checked,
            "ok": self.is_ok(),
            "issues": self.issues.iter().map(|issue| serde_json::json!({
                "type": issue.kind.as_key(),
                "target": issue.target,
                "detail": issue.detail,
            })).collect::<Vec<_>>(),
        })
    }
}

/// 对全项目记录运行检查。
#[must_use]
pub fn check_all(workspace: &Workspace) -> CheckReport {
    let records = workspace.index().records();
    let mut report = CheckReport {
        issues: Vec::new(),
        checked: records.len(),
    };

    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for record in records {
        if let Ok(id) = record.id() {
            *seen.entry(id).or_insert(0) += 1;
        }
    }
    for (id, count) in seen {
        if count > 1 {
            report.issues.push(Issue {
                kind: IssueKind::DuplicateId,
                target: id.to_string(),
                detail: format!("id `{id}` appears {count} times in this project"),
            });
        }
    }

    // 引用目标按 ID 建索引，引用校验因此是 O(n log n)。
    let known = workspace.index().by_id();

    for record in records {
        let Ok(id) = record.id() else {
            report.issues.push(Issue {
                kind: IssueKind::MissingField,
                target: record.path.display().to_string(),
                detail: "record has no `id` field".to_string(),
            });
            continue;
        };
        let target = id.to_string();
        if record.kind().is_err() {
            report.issues.push(Issue {
                kind: IssueKind::MissingField,
                target,
                detail: "record has no `kind` field".to_string(),
            });
            continue;
        }
        check_record(
            workspace.config(),
            record,
            &target,
            &known,
            &mut report.issues,
        );
    }

    report
}

/// 对单条记录运行检查，把问题追加到 `issues`。
pub fn check_record(
    config: &Config,
    record: &Record,
    target: &str,
    known_ids: &BTreeMap<String, &Record>,
    issues: &mut Vec<Issue>,
) {
    let Ok(kind_name) = record.kind() else {
        return;
    };

    let group = record.group();
    if group.is_empty() {
        issues.push(Issue {
            kind: IssueKind::MissingField,
            target: target.to_string(),
            detail: "record has no `group` field".to_string(),
        });
    } else if !config.has_group(group) {
        issues.push(Issue {
            kind: IssueKind::UnknownGroup,
            target: target.to_string(),
            detail: format!("group `{group}` is not declared in the project config"),
        });
    }

    let Some(kind) = config.kind(kind_name) else {
        issues.push(Issue {
            kind: IssueKind::UnknownKind,
            target: target.to_string(),
            detail: format!("kind `{kind_name}` is not declared in the project config"),
        });
        return;
    };

    for field in &kind.fields {
        match record.get(&field.name) {
            None if field.required => issues.push(Issue {
                kind: IssueKind::MissingField,
                target: target.to_string(),
                detail: format!("required field `{}` is missing", field.name),
            }),
            Some(value) if value.is_empty() && field.required => issues.push(Issue {
                kind: IssueKind::MissingField,
                target: target.to_string(),
                detail: format!("required field `{}` is empty", field.name),
            }),
            Some(value) if !value.is_empty() => {
                if let Err(error) = check_field_value(kind, field, &value.display()) {
                    issues.push(Issue {
                        kind: IssueKind::FieldValue,
                        target: target.to_string(),
                        detail: error.message().to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    for section in record.body.missing_required(&kind.required_sections) {
        issues.push(Issue {
            kind: IssueKind::MissingSection,
            target: target.to_string(),
            detail: format!("required section `{section}` is missing or empty"),
        });
    }

    for field in ["parent", "depends_on"] {
        for reference in record.references(field) {
            if reference == target {
                issues.push(Issue {
                    kind: IssueKind::BrokenReference,
                    target: target.to_string(),
                    detail: format!("`{field}` must not reference the record itself"),
                });
            } else if !known_ids.contains_key(&reference) {
                issues.push(Issue {
                    kind: IssueKind::BrokenReference,
                    target: target.to_string(),
                    detail: format!("`{field}` references unknown record `{reference}`"),
                });
            }
        }
    }

    let missing = crate::status::completion_gaps(config, record);
    if !missing.is_empty() {
        issues.push(Issue {
            kind: IssueKind::CompletionEvidence,
            target: target.to_string(),
            detail: format!(
                "completion value is recorded but is missing {}",
                missing.join(" and ")
            ),
        });
    }
}

// 完成依据的检查由 `crate::status::completion_gaps` 唯一判定；`check` 只把缺口呈现为检查发现。
