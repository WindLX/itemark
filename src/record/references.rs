//! Markdown 正文中的稳定记录引用。
//!
//! pulldown-cmark 的 source offsets 用来跳过代码块和行内代码；引用本身按源 Markdown
//! 扫描，因此别名和行号不会被渲染结果改写。

use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineReference {
    pub id: String,
    /// 从 Markdown 正文首行起算，1-based。
    pub line: usize,
}

/// 读取正文中的 `[[IM-N]]` 与 `[[IM-N|label]]`，跳过 Markdown 代码区。
#[must_use]
pub fn find_inline_references(markdown: &str) -> Vec<InlineReference> {
    let mut excluded = Vec::<Range<usize>>::new();
    let mut code_block_start = None;
    for (event, range) in Parser::new(markdown).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(_)) => code_block_start = Some(range.start),
            Event::End(TagEnd::CodeBlock) => {
                if let Some(start) = code_block_start.take() {
                    excluded.push(start..range.end);
                }
            }
            Event::Code(_) => excluded.push(range),
            _ => {}
        }
    }
    excluded.sort_by_key(|range| range.start);

    let mut references = Vec::new();
    let mut cursor = 0;
    let mut excluded_index = 0;
    while cursor < markdown.len() {
        let Some(relative_start) = markdown[cursor..].find("[[") else {
            break;
        };
        let start = cursor + relative_start;
        while excluded_index < excluded.len() && excluded[excluded_index].end <= start {
            excluded_index += 1;
        }
        if let Some(range) = excluded.get(excluded_index)
            && range.start <= start
            && start < range.end
        {
            cursor = range.end;
            continue;
        }

        let line_end = markdown[start..]
            .find('\n')
            .map_or(markdown.len(), |relative| start + relative);
        let Some(relative_end) = markdown[start + 2..line_end].find("]]") else {
            cursor = start + 2;
            continue;
        };
        let content_end = start + 2 + relative_end;
        let content = &markdown[start + 2..content_end];
        let id = content.split_once('|').map_or(content, |(id, _)| id);
        if is_item_id(id) {
            references.push(InlineReference {
                id: id.to_string(),
                line: markdown[..start]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1,
            });
        }
        cursor = content_end + 2;
    }
    references
}

fn is_item_id(id: &str) -> bool {
    id.strip_prefix("IM-").is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_keep_alias_and_body_line() {
        let references = find_inline_references("## 目标\n\ntext [[IM-2]]\n\nsee [[IM-3|label]]");
        assert_eq!(
            references,
            [
                InlineReference {
                    id: "IM-2".to_string(),
                    line: 3,
                },
                InlineReference {
                    id: "IM-3".to_string(),
                    line: 5,
                },
            ]
        );
    }

    #[test]
    fn fenced_and_inline_code_examples_are_ignored() {
        let references = find_inline_references(
            "[[IM-1]] `[[IM-2]]`\n\n```text\n[[IM-3]]\n```\n\n[[IM-4|actual]]",
        );
        assert_eq!(
            references,
            [
                InlineReference {
                    id: "IM-1".to_string(),
                    line: 1,
                },
                InlineReference {
                    id: "IM-4".to_string(),
                    line: 7,
                }
            ]
        );
    }
}
