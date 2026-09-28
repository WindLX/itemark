//! 默认分节名。
//!
//! 分节名是记录正文的约定词汇，不是可配置项：当前没有「按 kind 自定义分节名」的
//! 需求，因此这些名字只在项目语言里定义一次，记录、视图与命令层都引用这里。

/// 完成说明分节。
pub const COMPLETION: &str = "完成说明";

/// 证据分节。
pub const EVIDENCE: &str = "证据";

/// 进展分节。
pub const PROGRESS: &str = "进展";

/// 「下一步」分节的候选名，按优先级排列。
pub const NEXT_STEP: &[&str] = &["下一步", "当前下一步"];
