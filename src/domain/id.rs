//! 项目领域词汇：稳定 ID 与记录生命周期。
//!
//! 命令名、ID、配置键与 JSON 字段名是稳定标识，不随项目语言翻译。

use std::cmp::Ordering;

/// 记录 ID 前缀；ID 在项目内唯一，且不依赖文件路径。
pub const ID_PREFIX: &str = "IM-";
/// Legacy prefix accepted while repositories migrate existing references.
pub const LEGACY_ID_PREFIX: &str = "WL-";

/// 解析 ID 的序号部分，用于分配下一个 ID。
#[must_use]
pub fn id_number(id: &str) -> Option<u64> {
    let digits = id
        .strip_prefix(ID_PREFIX)
        .or_else(|| id.strip_prefix(LEGACY_ID_PREFIX))?;
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Compare valid Itemark IDs by their numeric suffix, with a lexical fallback for
/// equal numeric values and malformed IDs.
#[must_use]
pub fn compare_ids(left: &str, right: &str) -> Ordering {
    match (id_number(left), id_number(right)) {
        (Some(left_number), Some(right_number)) => {
            left_number.cmp(&right_number).then_with(|| left.cmp(right))
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => left.cmp(right),
    }
}

/// 按项目固定格式渲染 ID。
#[must_use]
pub fn format_id(number: u64) -> String {
    format!("{ID_PREFIX}{number}")
}

/// 记录生命周期：与业务状态相互独立，废弃只标记不删除。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Active,
    Dropped,
}

impl Lifecycle {
    #[must_use]
    pub fn is_dropped(self) -> bool {
        matches!(self, Self::Dropped)
    }

    #[must_use]
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Dropped => "dropped",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_through_the_project_format() {
        assert_eq!(format_id(1), "IM-1");
        assert_eq!(format_id(1234), "IM-1234");
        assert_eq!(id_number("IM-1"), Some(1));
        assert_eq!(id_number("WL-0001"), Some(1));
        assert_eq!(id_number("WL-12345"), Some(12345));
    }

    #[test]
    fn non_project_ids_have_no_number() {
        assert_eq!(id_number("WL-"), None);
        assert_eq!(id_number("IM-"), None);
        assert_eq!(id_number("WL-abcd"), None);
        assert_eq!(id_number("0001"), None);
        assert_eq!(id_number("WL-0001x"), None);
    }

    #[test]
    fn ids_compare_by_number_and_break_ties_lexically() {
        assert_eq!(compare_ids("IM-2", "IM-10"), Ordering::Less);
        assert_eq!(compare_ids("WL-0002", "IM-2"), Ordering::Greater);
        assert_eq!(compare_ids("bad", "IM-2"), Ordering::Greater);
    }
}
