//! 时间取值：生成时点、当前日期与日期解析。
//!
//! 统一使用 UTC，不读取系统 locale（项目语言由配置决定）。

use chrono::{NaiveDate, SecondsFormat, Utc};

/// 当前 UTC 时点的 RFC 3339 表示（秒精度、`Z` 结尾）。
#[must_use]
pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// 当前 UTC 日期，形如 `YYYY-MM-DD`。
#[must_use]
pub fn today() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// 校验 `YYYY-MM-DD` 并规范化为零填充形式；不是真实日期时返回 `None`。
#[must_use]
pub fn parse_date(text: &str) -> Option<String> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .ok()
        .map(|date| date.format("%Y-%m-%d").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 只测本层职责：把接受的日期规范化为零填充形式。
    /// 日期是否真实存在、格式是否合法由 chrono 负责，不在此重复测试。
    #[test]
    fn dates_are_zero_padded() {
        assert_eq!(parse_date("2024-2-9").as_deref(), Some("2024-02-09"));
    }
}
