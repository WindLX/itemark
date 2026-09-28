//! 项目领域词汇：稳定 ID 与记录生命周期。
//!
//! 命令名、ID、配置键与 JSON 字段名是稳定标识，不随项目语言翻译。

/// 记录 ID 前缀；ID 在项目内唯一，且不依赖文件路径。
pub const ID_PREFIX: &str = "WL-";

/// 解析 ID 的序号部分，用于分配下一个 ID。
#[must_use]
pub fn id_number(id: &str) -> Option<u64> {
    let digits = id.strip_prefix(ID_PREFIX)?;
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// 按项目固定格式渲染 ID。
#[must_use]
pub fn format_id(number: u64) -> String {
    format!("{ID_PREFIX}{number:04}")
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
        assert_eq!(format_id(1), "WL-0001");
        assert_eq!(format_id(1234), "WL-1234");
        assert_eq!(id_number("WL-0001"), Some(1));
        assert_eq!(id_number("WL-12345"), Some(12345));
    }

    #[test]
    fn non_project_ids_have_no_number() {
        assert_eq!(id_number("WL-"), None);
        assert_eq!(id_number("WL-abcd"), None);
        assert_eq!(id_number("0001"), None);
        assert_eq!(id_number("WL-0001x"), None);
    }
}
