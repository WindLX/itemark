//! Markdown 正文的分节处理。
//!
//! 正文由「前言」和若干二级标题分节组成。分节边界用 pulldown-cmark 解析，不自行
//! 猜测 `##` 行；分节内部保留原始 Markdown 文本。CLI 定向更新只改指定分节，其余
//! 内容按原顺序保留。

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BodyDoc {
    /// 首个二级标题之前的内容。
    pub preamble: String,
    pub sections: Vec<Section>,
}

impl BodyDoc {
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let text = normalise(text);
        let text = text.strip_prefix('\n').unwrap_or(&text);

        let mut headings: Vec<(String, Range<usize>)> = Vec::new();
        let mut heading_start: Option<usize> = None;
        for (event, range) in Parser::new(text).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    ..
                }) => {
                    heading_start = Some(range.start);
                }
                Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                    // `End` 事件的 range 覆盖整个标题块；取 range.end 作为分节正文起点。
                    let end = range.end;
                    if let Some(start) = heading_start.take()
                        && end > start
                        && end <= text.len()
                    {
                        // 直接取标题源码文本，保留 `code` 这类内联标记。
                        let literal = &text[start..end];
                        let name = literal
                            .trim_start_matches('#')
                            .trim_end_matches('#')
                            .trim()
                            .to_string();
                        headings.push((name, start..end));
                    }
                }
                _ => {}
            }
        }

        let mut headings: Vec<(String, Range<usize>)> = headings
            .into_iter()
            .filter(|(name, _)| !name.is_empty())
            .collect();

        let preamble = headings.first().map_or_else(
            || text.to_string(),
            |(_, range)| text[..range.start].to_string(),
        );

        for index in 0..headings.len() {
            let start = headings[index].1.end;
            let end = headings
                .get(index + 1)
                .map_or(text.len(), |(_, range)| range.start);
            headings[index].1 = start..end;
        }

        Self {
            preamble: trim_block(&preamble),
            sections: headings
                .into_iter()
                .map(|(title, range)| Section {
                    title,
                    body: trim_block(&text[range]),
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        if !self.preamble.trim().is_empty() {
            out.push_str(self.preamble.trim_end());
            out.push('\n');
        }
        for section in &self.sections {
            out.push('\n');
            out.push_str("## ");
            out.push_str(&section.title);
            out.push('\n');
            if !section.body.trim().is_empty() {
                out.push('\n');
                out.push_str(section.body.trim_end());
                out.push('\n');
            }
        }
        out
    }

    #[must_use]
    pub fn section(&self, title: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.title == title)
    }

    pub fn section_mut(&mut self, title: &str) -> Option<&mut Section> {
        self.sections
            .iter_mut()
            .find(|section| section.title == title)
    }

    #[must_use]
    pub fn has_section(&self, title: &str) -> bool {
        self.section(title).is_some()
    }

    /// 分节标题清单，保持文件顺序。
    #[must_use]
    pub fn titles(&self) -> Vec<String> {
        self.sections
            .iter()
            .map(|section| section.title.clone())
            .collect()
    }

    /// 写入一个分节：已有则整体替换，缺失则追加到末尾。
    pub fn set_section(&mut self, title: &str, body: &str) {
        let body = trim_block(body);
        match self.section_mut(title) {
            Some(section) => section.body = body,
            None => self.sections.push(Section {
                title: title.to_string(),
                body,
            }),
        }
    }

    /// 在分节末尾追加一行，保留已有内容；缺失则新建分节。
    pub fn append_line(&mut self, title: &str, line: &str) {
        match self.section_mut(title) {
            Some(section) => {
                let existing = section.body.trim_end();
                section.body = if existing.is_empty() {
                    line.to_string()
                } else {
                    format!("{existing}\n{line}")
                };
            }
            None => self.sections.push(Section {
                title: title.to_string(),
                body: line.to_string(),
            }),
        }
    }

    /// 必填分节中缺失或为空白的标题。
    #[must_use]
    pub fn missing_required(&self, required: &[String]) -> Vec<String> {
        required
            .iter()
            .filter(|title| {
                self.section(title)
                    .is_none_or(|section| section.body.trim().is_empty())
            })
            .cloned()
            .collect()
    }
}

fn trim_block(text: &str) -> String {
    text.trim_matches('\n').trim_end().to_string()
}

/// 去掉 BOM 并把 CRLF 统一成 LF；正文与头部两个入口共用同一次规范化。
fn normalise(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.replace("\r\n", "\n")
}

/// 把 `---` 包围的 YAML 头部与正文分开。
#[must_use]
pub fn split_front_matter(text: &str) -> Option<(String, String)> {
    let normalised = normalise(text);
    let rest = normalised.strip_prefix("---")?;
    if !rest.starts_with('\n') {
        return None;
    }
    let rest = &rest[1..];
    let mut end = None;
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches('\n');
        if trimmed == "---" || trimmed == "..." {
            end = Some(offset);
            break;
        }
        offset += line.len();
    }
    let end = end?;
    let front = rest[..end].to_string();
    let mut body = &rest[end..];
    body = body
        .strip_prefix("---")
        .or_else(|| body.strip_prefix("..."))
        .unwrap_or("");
    Some((front, body.trim_start_matches('\n').to_string()))
}

/// 拼装完整的记录文件文本。
#[must_use]
pub fn compose(front_matter: &str, body: &str) -> String {
    let mut out = String::from("---\n");
    out.push_str(front_matter.trim_end());
    out.push_str("\n---\n");
    let body = body.trim_start_matches('\n');
    if !body.trim().is_empty() {
        out.push('\n');
        out.push_str(body.trim_end());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_keep_file_order_and_skip_h1() {
        let doc = BodyDoc::parse("# 标题\n\n导语\n\n## 目标\n\n第一段\n\n## 进展\n\n第二段\n");
        assert_eq!(doc.titles(), ["目标", "进展"]);
        assert!(doc.preamble.contains("导语"));
        assert_eq!(doc.section("目标").expect("目标").body, "第一段");
    }

    #[test]
    fn inline_markup_in_a_heading_is_plain_text() {
        let doc = BodyDoc::parse("## 目标 `code`\n\n内容\n");
        assert_eq!(doc.titles(), ["目标 `code`"]);
    }

    #[test]
    fn set_section_replaces_only_that_section() {
        let mut doc = BodyDoc::parse("## 目标\n\n旧\n\n## 进展\n\n留\n");
        doc.set_section("目标", "新");
        assert_eq!(doc.render(), "\n## 目标\n\n新\n\n## 进展\n\n留\n");
    }

    #[test]
    fn append_line_keeps_existing_progress() {
        let mut doc = BodyDoc::parse("## 进展\n\n- 2026-01-01：第一条\n");
        doc.append_line("进展", "- 2026-01-02：第二条");
        let body = doc.section("进展").expect("进展").body.clone();
        assert!(body.contains("第一条"));
        assert!(body.trim_end().ends_with("第二条"));
    }

    #[test]
    fn append_line_creates_a_missing_section() {
        let mut doc = BodyDoc::parse("## 目标\n\n内容\n");
        doc.append_line("进展", "- 一条");
        assert_eq!(doc.titles(), ["目标", "进展"]);
    }

    #[test]
    fn missing_required_reports_blank_sections() {
        let doc = BodyDoc::parse("## 目标\n\n\n## 证据\n\n有\n");
        assert_eq!(
            doc.missing_required(&["目标".into(), "证据".into()]),
            ["目标"]
        );
    }

    #[test]
    fn front_matter_split_accepts_both_terminators() {
        let (front, body) = split_front_matter("---\nid: WL-0001\n---\n\n正文\n").expect("split");
        assert_eq!(front, "id: WL-0001\n");
        assert_eq!(body, "正文\n");
        assert!(split_front_matter("没有头部\n").is_none());
    }

    #[test]
    fn compose_round_trips() {
        let text = "---\nid: WL-0001\n---\n\n## 目标\n\n内容\n";
        let (front, body) = split_front_matter(text).expect("split");
        assert_eq!(compose(&front, &body), text);
    }
}
