//! 人读文本与机器可读 JSON 的呈现。
//!
//! 同一操作共享行为，只改变呈现方式；命令名、ID、配置键与 JSON 键保持稳定，不随
//! 项目语言翻译。

use crate::error::Result;

/// 输出模式：默认人读文本，`--json` 换机器可读呈现。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Text,
    Json,
}

impl OutputMode {
    #[must_use]
    pub fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }

    /// 从 `--json` 标志取值；默认人读文本。
    #[must_use]
    pub fn from_json(json: bool) -> Self {
        if json { Self::Json } else { Self::Text }
    }
}

/// 打印机器可读 JSON。
pub fn print_json(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// 人读输出文案访问器，值由随程序打包的静态 JSON 词典提供。
pub struct Labels {
    english: bool,
}

impl Labels {
    fn get(&self, key: &str) -> &'static str {
        crate::i18n::text(key, if self.english { "en" } else { "zh-CN" })
    }
}

macro_rules! labels {
    ($($field:ident => $key:literal;)*) => {
        impl Labels { $(pub fn $field(&self) -> &'static str { self.get($key) })* }
    };
}

labels! {
    id => "id"; kind => "kind"; group => "group"; title => "title"; status => "status";
    parent => "parent"; depends_on => "depends_on"; completion => "completion";
    completion_note => "completion_note"; completion_evidence => "completion_evidence";
    body => "body"; lifecycle => "lifecycle"; active => "active"; dropped => "dropped";
    items => "items"; records => "records"; overview => "overview"; handoff_title => "handoff_title";
    generated_at => "generated_at"; sources => "sources"; in_progress => "in_progress";
    blocked => "blocked"; unverified => "unverified"; todo => "todo"; done => "done";
    no_status => "no_status"; next_step => "next_step"; dropped_items => "dropped_items";
    groups => "groups"; kinds => "kinds"; fields => "fields"; required_sections => "required_sections";
    template => "template"; description => "description"; issues => "issues"; warnings => "warnings"; checked => "checked";
    checked_kinds => "checked_kinds"; ok => "ok"; saved => "saved"; updated => "updated";
    created => "created"; init_line => "init_line"; next_step_hint => "next_step_hint";
    kind_summary => "kind_summary"; group_summary => "group_summary"; template_line => "template_line";
    present => "present"; missing => "missing"; required_marker => "required_marker";
    list_separator => "list_separator"; dropped_note => "dropped_note"; line => "line"; heading => "heading";
    record_line => "record_line"; log_note => "log_note"; body_block => "body_block";
    heading_count => "heading_count"; counts_paren => "counts_paren"; indented_line => "indented_line";
    source_line => "source_line"; issue_line => "issue_line"; group_created => "group_created";
    field_line => "field_line"; no_matches => "no_matches"; list_count => "list_count";
    list_filter => "list_filter"; group_filter => "group_filter"; kind_filter => "kind_filter";
    status_filter => "status_filter"; all_filter => "all_filter";
}

/// 用语言相关的句子模板渲染人读文本：模板里的 `{}` 按顺序被 `parts` 填充。
/// 文案来自随二进制打包的 JSON 词典。
#[must_use]
pub fn fill(template: &str, parts: &[&str]) -> String {
    let mut rendered = String::with_capacity(template.len() + 16);
    let mut rest = template;
    let mut parts = parts.iter();
    while let Some(index) = rest.find("{}") {
        rendered.push_str(&rest[..index]);
        match parts.next() {
            Some(part) => rendered.push_str(part),
            None => rendered.push_str("{}"),
        }
        rest = &rest[index + 2..];
    }
    rendered.push_str(rest);
    rendered
}

#[must_use]
pub fn labels(language: &str) -> Labels {
    Labels {
        english: crate::i18n::is_english(language),
    }
}

/// 生命周期取值在输出中的呈现。
#[must_use]
pub fn lifecycle_text(language: &str, dropped: bool) -> &'static str {
    let labels = labels(language);
    if dropped {
        labels.dropped()
    } else {
        labels.active()
    }
}
