//! kind 的模板渲染与字段取值校验。
//!
//! kind 由项目配置与 Markdown 模板维护，声明名称、说明、模板、少量字段约束和必填
//! 分节。规则保持声明式：没有脚本、条件语言、状态迁移或工作流引擎。

use std::collections::BTreeMap;

use crate::error::{Result, WorkspaceError};
use crate::record::markdown::BodyDoc;
use crate::workspace::config::{Config, FieldDef, KindConfig};

/// 用 `{{字段}}` 占位符渲染模板文本；未知占位符保留原样。
#[must_use]
pub fn render_template(template: &str, values: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let key = after[..end].trim();
        match values.get(key) {
            Some(value) => out.push_str(value),
            None => {
                out.push_str("{{");
                out.push_str(&after[..end]);
                out.push_str("}}");
            }
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// 占位符取值表：ID、kind、group、title 与 kind 声明字段的模板默认值。
#[must_use]
pub fn template_values(
    kind: &KindConfig,
    values: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for field in &kind.fields {
        map.insert(field.name.clone(), String::new());
    }
    map.extend(
        values
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    map
}

/// 读取 kind 模板的分节骨架；kind 未声明模板或模板缺失时返回 `None`。
pub fn template_body(config: &Config, kind: &KindConfig) -> Result<Option<BodyDoc>> {
    let Some(path) = config.template_path(kind) else {
        return Ok(None);
    };
    if !path.is_file() {
        return Err(WorkspaceError::runtime(format!(
            "kind template is missing: {}",
            path.display()
        ))
        .at(&path));
    }
    let text = std::fs::read_to_string(&path).map_err(|error| {
        WorkspaceError::runtime(format!("cannot read kind template: {error}")).at(&path)
    })?;
    let body = crate::record::markdown::split_front_matter(&text).map_or(text, |(_, body)| body);
    Ok(Some(BodyDoc::parse(&body)))
}

/// 字段类型的唯一词表。
///
/// 声明校验（配置加载）、取值校验（写入与 `check`）与 `kind check` 都从这里取，
/// 新增或修改一种类型只改这个模块。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String,
    Int,
    Bool,
    Enum,
    Date,
}

impl FieldType {
    /// 全部可用类型，顺序即错误信息里的呈现顺序。
    pub const ALL: [Self; 5] = [Self::String, Self::Int, Self::Bool, Self::Enum, Self::Date];

    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.as_str() == name)
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Enum => "enum",
            Self::Date => "date",
        }
    }

    /// 只有 `enum` 用 `values` 声明允许取值，其他类型声明 `values` 是配置错误。
    #[must_use]
    pub fn takes_values(self) -> bool {
        self == Self::Enum
    }

    /// 供错误信息使用的类型清单。
    #[must_use]
    pub fn available() -> String {
        Self::ALL
            .iter()
            .map(|field_type| field_type.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// 按字段类型与枚举取值校验字段值；空值由调用方按 required 处理。
pub fn check_field_value(kind: &KindConfig, field: &FieldDef, value: &str) -> Result<()> {
    let value = value.trim();
    let invalid = |expected: &str| {
        WorkspaceError::usage(format!(
            "kind `{}` field `{}` expects {expected}, got `{value}`",
            kind.name, field.name
        ))
    };
    let Some(field_type) = FieldType::parse(&field.field_type) else {
        return Err(WorkspaceError::usage(format!(
            "kind `{}` field `{}` has unsupported type `{}`",
            kind.name, field.name, field.field_type
        )));
    };
    match field_type {
        FieldType::String => Ok(()),
        FieldType::Date => {
            if is_iso_date(value) {
                Ok(())
            } else {
                Err(invalid("a real YYYY-MM-DD date"))
            }
        }
        FieldType::Int => value
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| invalid("an integer")),
        FieldType::Bool => match value {
            "true" | "false" => Ok(()),
            _ => Err(invalid("true or false")),
        },
        FieldType::Enum => {
            if field.values.iter().any(|allowed| allowed == value) {
                Ok(())
            } else {
                Err(invalid(&format!("one of {}", field.values.join(", "))))
            }
        }
    }
}

/// 校验日期字段：形状为 `YYYY-MM-DD`，且是真实存在的日期。
#[must_use]
pub fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    let shape = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    shape && crate::time::parse_date(value).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_placeholders_are_substituted() {
        let mut values = BTreeMap::new();
        values.insert("id".to_string(), "WL-0001".to_string());
        values.insert("title".to_string(), "标题".to_string());
        let text = render_template("---\nid: \"{{id}}\"\ntitle: \"{{title}}\"\n---\n", &values);
        assert!(text.contains("WL-0001"));
        assert!(text.contains("标题"));
    }

    #[test]
    fn unknown_placeholders_are_left_alone() {
        let values = BTreeMap::new();
        assert_eq!(render_template("{{unknown}}", &values), "{{unknown}}");
    }

    #[test]
    fn enum_values_are_enforced() {
        let kind = KindConfig {
            name: "work".into(),
            description: String::new(),
            template: None,
            required_sections: Vec::new(),
            completion_field: None,
            completion_values: Vec::new(),
            fields: Vec::new(),
        };
        let field = FieldDef {
            name: "status".into(),
            field_type: "enum".into(),
            required: true,
            values: vec!["todo".into()],
        };
        assert!(check_field_value(&kind, &field, "todo").is_ok());
        assert!(check_field_value(&kind, &field, "done").is_err());
    }

    #[test]
    fn dates_are_calendar_checked() {
        assert!(is_iso_date("2026-01-31"));
        assert!(!is_iso_date("2026-1-31"));
        assert!(!is_iso_date("2026-13-45"));
    }
}
