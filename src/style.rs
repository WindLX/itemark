//! 终端着色。
//!
//! 是否输出 ANSI 转义只在这里决定：`--color` 显式指定优先；`auto` 要求标准输出是终端
//! 且未设置 `NO_COLOR`；JSON 输出永不着色。样式也集中在本模块，命令层只调用
//! [`paint`]，不自己拼转义序列，以便整体关闭时输出保持纯文本。

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

use anstyle::{AnsiColor, Effects, Style};

/// `--color` 的取值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Choice {
    /// 终端支持且未设置 `NO_COLOR` 时着色。
    #[default]
    Auto,
    /// 总是着色，忽略 `NO_COLOR` 与终端检测。
    Always,
    /// 从不着色。
    Never,
}

impl Choice {
    /// 解析 `--color` 的取值；不认识时返回 `None`，由参数层报错。
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }

    /// 该取值下是否输出 ANSI 转义。
    #[must_use]
    pub fn wants_color(self, is_terminal: bool, no_color: bool) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => is_terminal && !no_color,
        }
    }
}

/// `NO_COLOR` 只要被设置为非空值就表示使用者不要颜色。
#[must_use]
pub fn no_color_requested() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
}

/// 标准输出是否为终端。
#[must_use]
pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

static ENABLED: AtomicBool = AtomicBool::new(false);

/// 按 `--color`、终端与 `NO_COLOR` 决定是否着色；进程启动时设置一次。
pub fn enable(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// 当前是否输出着色文本。
#[must_use]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// 按样式包裹文本；着色关闭或文本为空时原样返回。
#[must_use]
pub fn paint(style: Style, text: &str) -> String {
    if !enabled() || text.is_empty() {
        return text.to_string();
    }
    format!("{style}{text}{style:#}")
}

/// 小标题与分节名。
#[must_use]
pub fn heading() -> Style {
    Style::new().effects(Effects::BOLD)
}

/// 字段名等次要标签。
#[must_use]
pub fn label() -> Style {
    Style::new().effects(Effects::DIMMED)
}

/// 强调的命令名、ID 与计数。
#[must_use]
pub fn accent() -> Style {
    Style::new()
        .fg_color(Some(AnsiColor::Cyan.into()))
        .effects(Effects::BOLD)
}

/// 成功结果。
#[must_use]
pub fn ok() -> Style {
    Style::new().fg_color(Some(AnsiColor::Green.into()))
}

/// 需要注意但不致命的结果。
#[must_use]
pub fn warn() -> Style {
    Style::new().fg_color(Some(AnsiColor::Yellow.into()))
}

/// 失败与冲突。
#[must_use]
pub fn error() -> Style {
    Style::new()
        .fg_color(Some(AnsiColor::Red.into()))
        .effects(Effects::BOLD)
}

/// 次要说明、路径与来源。
#[must_use]
pub fn muted() -> Style {
    Style::new().fg_color(Some(AnsiColor::BrightBlack.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_auto_consults_the_terminal_and_no_color() {
        assert!(Choice::Always.wants_color(false, true));
        assert!(!Choice::Never.wants_color(true, false));
        assert!(Choice::Auto.wants_color(true, false));
        assert!(!Choice::Auto.wants_color(false, false));
        assert!(!Choice::Auto.wants_color(true, true));
    }

    #[test]
    fn parse_accepts_only_the_three_spellings() {
        assert_eq!(Choice::parse("auto"), Some(Choice::Auto));
        assert_eq!(Choice::parse("always"), Some(Choice::Always));
        assert_eq!(Choice::parse("never"), Some(Choice::Never));
        assert_eq!(Choice::parse("AUTO"), None);
        assert_eq!(Choice::parse(""), None);
    }

    #[test]
    fn paint_adds_escapes_only_while_enabled() {
        let previous = enabled();

        enable(false);
        assert_eq!(paint(ok(), "done"), "done");

        enable(true);
        let painted = paint(ok(), "done");
        assert!(painted.starts_with('\u{1b}'), "{painted:?}");
        assert!(painted.ends_with("\u{1b}[0m"), "{painted:?}");
        assert_eq!(paint(heading(), ""), "");

        enable(previous);
    }
}
