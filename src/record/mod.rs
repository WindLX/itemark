//! 一条记录：YAML 头部字段 + Markdown 正文。
//!
//! 记录文件是权威来源，ID 独立于文件路径。读取时保留完整原文用于写入前比对；
//! 头部字段与正文分节都可定向修改，其余内容原样保留。

pub mod index;
pub mod markdown;

use std::path::{Path, PathBuf};

use crate::domain::{Field, Lifecycle, Scalar, front_matter, section};
use crate::error::{Result, WorkspaceError, read_error};

pub use markdown::BodyDoc;

/// 已从磁盘加载的记录。
#[derive(Debug, Clone)]
pub struct Record {
    pub path: PathBuf,
    /// 读取时刻的完整文件内容，用于写入前比对冲突。
    pub raw: String,
    pub fields: Vec<Field>,
    pub body: BodyDoc,
}

/// 一条待创建记录的内容。
#[derive(Debug, Clone, Default)]
pub struct NewRecord {
    pub fields: Vec<Field>,
    pub body: BodyDoc,
}

impl Record {
    pub fn read(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path).map_err(|error| read_error(path, error))?;
        Self::parse(path, raw)
    }

    pub fn parse(path: &Path, raw: String) -> Result<Self> {
        let (front, body_text) = markdown::split_front_matter(&raw)
            .ok_or_else(|| WorkspaceError::runtime("缺少 `---` 包围的 YAML 头部").at(path))?;
        let fields = front_matter::parse(&front, path)?;
        Ok(Self {
            path: path.to_path_buf(),
            raw,
            fields,
            body: BodyDoc::parse(&body_text),
        })
    }

    pub fn id(&self) -> Result<&str> {
        self.text_field("id")
            .ok_or_else(|| WorkspaceError::runtime("记录缺少 `id` 字段").at(&self.path))
    }

    pub fn kind(&self) -> Result<&str> {
        self.text_field("kind")
            .ok_or_else(|| WorkspaceError::runtime("记录缺少 `kind` 字段").at(&self.path))
    }

    pub fn group(&self) -> &str {
        self.text_field("group").unwrap_or("")
    }

    pub fn title(&self) -> &str {
        self.text_field("title").unwrap_or("")
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Scalar> {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    /// 生命周期：`dropped: true` 表示已废弃；字段缺失视为有效。
    #[must_use]
    pub fn lifecycle(&self) -> Lifecycle {
        match self.get("dropped") {
            Some(Scalar::Bool(true)) => Lifecycle::Dropped,
            Some(Scalar::Text(text)) if text == "true" => Lifecycle::Dropped,
            _ => Lifecycle::Active,
        }
    }

    /// 业务状态：由 kind 的完成字段承载，CLI 不硬编码状态名。
    #[must_use]
    pub fn status(&self, config: &crate::workspace::config::Config) -> Option<String> {
        let kind = self.kind().ok().and_then(|name| config.kind(name))?;
        let field = kind.status_field()?;
        self.get(field).map(Scalar::display)
    }

    /// 父项 ID。
    #[must_use]
    pub fn parent(&self) -> Option<String> {
        let value = self.get("parent")?.display();
        (!value.trim().is_empty()).then(|| value.trim().to_string())
    }

    /// 前置依赖 ID 列表。
    #[must_use]
    pub fn depends_on(&self) -> Vec<String> {
        self.references("depends_on")
    }

    /// 引用字段：单值字段按一个 ID，列表字段按多个 ID；空值忽略。
    #[must_use]
    pub fn references(&self, field: &str) -> Vec<String> {
        self.get(field).map(Scalar::items).unwrap_or_default()
    }

    /// 完成说明：优先取头部 `completion_note`，否则取「完成说明」分节。
    #[must_use]
    pub fn completion_note(&self) -> String {
        self.get("completion_note")
            .and_then(Scalar::as_text)
            .map(str::to_string)
            .filter(|text| !text.trim().is_empty())
            .or_else(|| {
                self.body
                    .section(section::COMPLETION)
                    .map(|section| section.body.clone())
            })
            .unwrap_or_default()
    }

    /// 证据：优先取头部 `completion_evidence`，否则取「证据」分节。
    #[must_use]
    pub fn completion_evidence(&self) -> String {
        self.get("completion_evidence")
            .and_then(Scalar::as_text)
            .map(str::to_string)
            .filter(|text| !text.trim().is_empty())
            .or_else(|| {
                self.body
                    .section(section::EVIDENCE)
                    .map(|section| section.body.clone())
            })
            .unwrap_or_default()
    }

    fn text_field(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(Scalar::as_text)
    }
}

impl NewRecord {
    /// 用模板渲染结果与头部字段拼出记录文本。
    #[must_use]
    pub fn render(&self) -> String {
        let front = front_matter::render(&self.fields);
        markdown::compose(&front, &self.body.render())
    }
}

impl Record {
    /// 在当前字段顺序上重排为规范顺序，并把未识别的字段留在末尾。
    pub fn set(&mut self, name: &str, value: Scalar) {
        match self.fields.iter_mut().find(|(key, _)| key == name) {
            Some(slot) => slot.1 = value,
            None => self.fields.push((name.to_string(), value)),
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.fields.retain(|(key, _)| key != name);
    }

    /// 渲染完整记录文件文本。
    #[must_use]
    pub fn render(&self) -> String {
        let front = front_matter::render(&self.fields);
        markdown::compose(&front, &self.body.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Record {
        Record::parse(Path::new("WL-0001.md"), text.to_string()).expect("parse record")
    }

    #[test]
    fn missing_front_matter_is_reported() {
        let error = Record::parse(Path::new("WL-0001.md"), "没有头部\n".to_string())
            .expect_err("missing header");
        assert!(error.message().contains("---"));
    }

    #[test]
    fn lifecycle_is_dropped_only_when_marked() {
        assert_eq!(
            parse("---\nid: WL-0001\n---\n").lifecycle(),
            Lifecycle::Active
        );
        assert_eq!(
            parse("---\nid: WL-0001\ndropped: true\n---\n").lifecycle(),
            Lifecycle::Dropped
        );
    }

    #[test]
    fn references_read_single_and_list_values() {
        let record = parse("---\nid: WL-0002\nparent: WL-0001\ndepends_on: WL-0003\n---\n");
        assert_eq!(record.parent().as_deref(), Some("WL-0001"));
        assert_eq!(record.depends_on(), ["WL-0003"]);
        assert!(
            parse("---\nid: WL-0001\nparent: \"\"\n---\n")
                .parent()
                .is_none()
        );
    }

    #[test]
    fn completion_falls_back_to_body_sections() {
        let record = parse("---\nid: WL-0001\n---\n## 完成说明\n\n已交付\n\n## 证据\n\n测试输出\n");
        assert_eq!(record.completion_note(), "已交付");
        assert_eq!(record.completion_evidence(), "测试输出");
    }

    #[test]
    fn header_completion_wins_over_body() {
        let record = parse("---\nid: WL-0001\ncompletion_note: 头部\n---\n## 完成说明\n\n正文\n");
        assert_eq!(record.completion_note(), "头部");
    }

    #[test]
    fn render_round_trips_a_record() {
        let text = "---\nid: WL-0001\nkind: work\n---\n\n## 目标\n\n交付 CLI\n";
        assert_eq!(parse(text).render(), text);
    }
}
