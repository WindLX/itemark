//! YAML 头部字段值：项目自己的取值形态与引用规则。
//!
//! 字段可以是标量，也可以是标量列表（如 `depends_on: [WL-0004, WL-0008]`）。
//! 取值与 YAML 之间的转换集中在 [`Scalar::from_yaml`] 与 [`Scalar::to_yaml`]，
//! 其余模块只接触 [`Scalar`]，不关心底层 YAML 库。

use std::path::Path;

use serde_norway::Value as YamlValue;

use crate::error::{Result, WorkspaceError};

/// 头部字段值。
#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    Text(String),
    Int(i64),
    Bool(bool),
    List(Vec<Self>),
    Null,
}

impl Scalar {
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// 头部字段在 CLI 输出与字段设置中使用的字符串形态。
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Int(value) => value.to_string(),
            Self::Bool(value) => value.to_string(),
            Self::List(items) => items
                .iter()
                .map(Self::display)
                .collect::<Vec<_>>()
                .join(", "),
            Self::Null => String::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(text) => text.trim().is_empty(),
            Self::Null => true,
            Self::List(items) => items.iter().all(Self::is_empty),
            Self::Int(_) | Self::Bool(_) => false,
        }
    }

    /// 列表取值按 ID 等文本逐项取出；标量取值按单项处理。
    #[must_use]
    pub fn items(&self) -> Vec<String> {
        match self {
            Self::List(items) => items.iter().map(Self::display).collect(),
            other => {
                let text = other.display();
                if text.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![text.trim().to_string()]
                }
            }
        }
    }

    /// 未加引号的标量文本；引用值（如 `WL-0001`）按文本处理。
    #[must_use]
    pub fn from_plain(text: &str) -> Self {
        match text {
            "null" | "~" => Self::Null,
            "true" => Self::Bool(true),
            "false" => Self::Bool(false),
            other => match other.parse::<i64>() {
                Ok(value) => Self::Int(value),
                Err(_) => Self::Text(other.to_string()),
            },
        }
    }

    /// 把 YAML 解析结果收敛为字段值。
    pub fn from_yaml(value: &YamlValue, path: &Path) -> Result<Self> {
        match value {
            YamlValue::Null => Ok(Self::Null),
            YamlValue::Bool(flag) => Ok(Self::Bool(*flag)),
            YamlValue::Number(number) => Ok(number
                .as_i64()
                .map_or_else(|| Self::Text(number.to_string()), Self::Int)),
            YamlValue::String(text) => Ok(Self::Text(text.clone())),
            YamlValue::Sequence(items) => items
                .iter()
                .map(|item| Self::from_yaml(item, path))
                .collect::<Result<Vec<_>>>()
                .map(Self::List),
            YamlValue::Mapping(_) | YamlValue::Tagged(_) => Err(WorkspaceError::runtime(
                "header fields must be scalars or lists of scalars",
            )
            .at(path)),
        }
    }

    /// 字段值的 YAML 形态。
    #[must_use]
    pub fn to_yaml(&self) -> YamlValue {
        match self {
            Self::Text(text) => YamlValue::String(text.clone()),
            Self::Int(value) => YamlValue::Number((*value).into()),
            Self::Bool(flag) => YamlValue::Bool(*flag),
            Self::List(items) => {
                YamlValue::Sequence(items.iter().map(Self::to_yaml).collect::<Vec<_>>())
            }
            Self::Null => YamlValue::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path() -> PathBuf {
        PathBuf::from("WL-0001.md")
    }

    #[test]
    fn plain_scalars_take_their_yaml_type() {
        assert_eq!(
            Scalar::from_plain("WL-0001"),
            Scalar::Text("WL-0001".into())
        );
        assert_eq!(Scalar::from_plain("42"), Scalar::Int(42));
        assert_eq!(Scalar::from_plain("true"), Scalar::Bool(true));
        assert_eq!(Scalar::from_plain("null"), Scalar::Null);
    }

    #[test]
    fn ids_are_never_read_as_numbers() {
        assert_eq!(Scalar::from_plain("WL-0001").display(), "WL-0001");
        assert!(!Scalar::from_plain("WL-0001").is_empty());
    }

    #[test]
    fn yaml_values_keep_their_type_and_list_shape() {
        let value: YamlValue = serde_norway::from_str("depends_on: [WL-0004, WL-0008]").unwrap();
        let list = value
            .as_mapping()
            .and_then(|map| map.get(YamlValue::String("depends_on".into())))
            .unwrap();
        assert_eq!(
            Scalar::from_yaml(list, &path()).expect("list"),
            Scalar::List(vec![
                Scalar::Text("WL-0004".into()),
                Scalar::Text("WL-0008".into())
            ])
        );
    }

    #[test]
    fn nested_mappings_are_not_field_values() {
        let value: YamlValue = serde_norway::from_str("meta:\n  a: 1\n").unwrap();
        let nested = value
            .as_mapping()
            .and_then(|map| map.get(YamlValue::String("meta".into())))
            .unwrap();
        assert!(Scalar::from_yaml(nested, &path()).is_err());
    }

    #[test]
    fn list_items_are_read_as_text_ids() {
        let list = Scalar::List(vec![
            Scalar::Text("WL-0004".into()),
            Scalar::Text("WL-0008".into()),
        ]);
        assert_eq!(list.items(), ["WL-0004", "WL-0008"]);
        assert_eq!(Scalar::Text("WL-0004".into()).items(), ["WL-0004"]);
        assert!(Scalar::Null.items().is_empty());
    }
}
