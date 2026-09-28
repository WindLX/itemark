//! 记录文件的头部：`---` 包围的 YAML 字段序列。
//!
//! 头部由 YAML 库（serde_norway）解析与渲染，字段顺序按文件顺序保留，包括未知键。
//! 记录头部只使用 YAML 的扁平形态：标量或标量列表；嵌套映射、锚点与自定义标签会被拒绝，
//! 以保持头部可直接阅读、可逐字段定位。写入时保持既有顺序，只改动目标字段。

use std::path::Path;

use serde_norway::{Mapping, Value as YamlValue};

use crate::domain::Field;
use crate::domain::scalar::Scalar;
use crate::error::{Result, WorkspaceError};

/// 解析头部文本为字段序列，保留文件顺序。
pub fn parse(front: &str, path: &Path) -> Result<Vec<Field>> {
    if front.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: YamlValue = serde_norway::from_str(front).map_err(|error| {
        WorkspaceError::runtime(format!("cannot parse record header as YAML: {error}")).at(path)
    })?;
    if value.is_null() {
        return Ok(Vec::new());
    }
    let YamlValue::Mapping(mapping) = value else {
        return Err(WorkspaceError::runtime("record header must be a mapping of fields").at(path));
    };
    let mut fields = Vec::with_capacity(mapping.len());
    for (key, value) in &mapping {
        let YamlValue::String(key) = key else {
            return Err(WorkspaceError::runtime("record header keys must be strings").at(path));
        };
        fields.push((key.clone(), Scalar::from_yaml(value, path)?));
    }
    Ok(fields)
}

/// 渲染字段序列为头部文本（不含 `---` 分隔行）。
#[must_use]
pub fn render(fields: &[Field]) -> String {
    let mut mapping = Mapping::new();
    for (key, value) in fields {
        mapping.insert(YamlValue::String(key.clone()), value.to_yaml());
    }
    // 只含标量与标量列表的映射一定能序列化。
    serde_norway::to_string(&YamlValue::Mapping(mapping)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path() -> PathBuf {
        PathBuf::from("WL-0001.md")
    }

    #[test]
    fn fields_keep_their_file_order() {
        let fields = parse("id: WL-0001\nkind: work\ntitle: 记录\n", &path()).expect("parse");
        let keys: Vec<&str> = fields.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["id", "kind", "title"]);
        assert_eq!(fields[0].1, Scalar::Text("WL-0001".into()));
    }

    #[test]
    fn unknown_keys_survive_a_round_trip() {
        let text = "id: WL-0001\ncustom: 保留\n";
        let fields = parse(text, &path()).expect("parse");
        assert_eq!(render(&fields), text);
    }

    /// YAML 列表语法（流式 / 块式 / 空列表）由 serde_norway 负责，此处只测
    /// 本项目自己的契约：列表能读成 `Scalar::List` 并原样写回。
    #[test]
    fn lists_survive_a_round_trip() {
        let text = "id: WL-0003\ndepends_on:\n- WL-0004\n- WL-0008\n";
        let fields = parse(text, &path()).expect("parse");
        assert_eq!(render(&fields), text);
    }

    #[test]
    fn nested_headers_are_reported() {
        let error =
            parse("id: WL-0001\nmeta:\n  a: 1\n", &path()).expect_err("nesting is rejected");
        assert!(!error.message().is_empty());
    }

    #[test]
    fn non_mapping_headers_are_reported() {
        let error = parse("- WL-0001\n", &path()).expect_err("sequence is rejected");
        assert!(!error.message().is_empty());
    }

    #[test]
    fn empty_header_parses_to_no_fields() {
        assert!(parse("", &path()).expect("parse").is_empty());
        assert!(parse("   \n", &path()).expect("parse").is_empty());
    }
}
