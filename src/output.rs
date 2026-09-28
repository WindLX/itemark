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

/// 人读输出的栏目标签；本库默认简体中文。
pub struct Labels(&'static str);

macro_rules! labels {
    ($($field:ident => $zh:literal, $en:literal;)*) => {
        impl Labels {
            $(pub fn $field(&self) -> &'static str {
                match self.0 {
                    "en" => $en,
                    _ => $zh,
                }
            })*
        }
    };
}

labels! {
    id => "ID", "ID";
    kind => "kind", "kind";
    group => "group", "group";
    title => "标题", "title";
    status => "状态", "status";
    parent => "父项", "parent";
    depends_on => "前置依赖", "depends_on";
    completion => "完成依据", "completion";
    completion_note => "完成说明", "completion note";
    completion_evidence => "证据", "evidence";
    body => "正文", "body";
    lifecycle => "生命周期", "lifecycle";
    active => "有效", "active";
    dropped => "已废弃", "dropped";
    items => "记录", "items";
    records => "记录数", "records";
    overview => "当前总览", "current overview";
    handoff_title => "交接摘要", "handoff";
    generated_at => "生成时点", "generated at";
    sources => "来源记录", "sources";
    in_progress => "进行中", "in progress";
    blocked => "阻塞", "blocked";
    unverified => "完成但未验证", "done but unverified";
    todo => "待办", "todo";
    done => "已完成", "done";
    no_status => "无业务状态", "no status";
    next_step => "下一步", "next step";
    dropped_items => "已废弃记录", "dropped records";
    groups => "分组", "groups";
    kinds => "kind 定义", "kinds";
    fields => "字段", "fields";
    required_sections => "必填分节", "required sections";
    template => "模板", "template";
    description => "说明", "description";
    issues => "问题", "issues";
    checked => "检查记录数", "records checked";
    checked_kinds => "检查 kind 数", "kinds checked";
    ok => "检查通过", "check passed";
    saved => "已保存", "saved";
    updated => "已更新", "updated";
    created => "已创建", "created";
}

#[must_use]
pub fn labels(language: &str) -> Labels {
    if language.eq_ignore_ascii_case("en") || language.to_lowercase().starts_with("en-") {
        Labels("en")
    } else {
        Labels("zh")
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
