//! 现场总览与交接视图。
//!
//! 每次查询都从当前记录、group、kind、生命周期与业务状态现场构建，不使用持久索引。
//! 输出带上来源记录 ID 与生成时点；默认只读，仅显式选项才写入摘要快照。

use crate::output::{fill, labels, print_json};
use crate::record::Record;
use crate::workspace::Workspace;

/// 记录的业务状态来自 [`crate::status::of`]，总览只做分组呈现。
pub use crate::status::State;

/// 总览中的一条记录。
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub group: String,
    pub parent: Option<String>,
    pub depends_on: Vec<String>,
    pub state: State,
    pub next_step: Option<String>,
    pub dropped: bool,
}

/// 总览：记录清单、group/kind 结构与生成时点。
#[derive(Debug, Clone)]
pub struct Overview {
    pub entries: Vec<Entry>,
    pub groups: Vec<String>,
    pub kinds: Vec<String>,
    pub generated_at: String,
}

impl Overview {
    #[must_use]
    pub fn count_of(&self, state: State) -> usize {
        self.entries
            .iter()
            .filter(|entry| !entry.dropped && entry.state == state)
            .count()
    }

    #[must_use]
    pub fn source_ids(&self) -> Vec<&str> {
        self.entries.iter().map(|entry| entry.id.as_str()).collect()
    }

    /// 机器可读形态；键名稳定，不随项目语言变化。
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "generated_at": self.generated_at,
            "groups": self.groups,
            "kinds": self.kinds,
            "counts": {
                "todo": self.count_of(State::Todo),
                "in_progress": self.count_of(State::InProgress),
                "blocked": self.count_of(State::Blocked),
                "done_unverified": self.count_of(State::Unverified),
                "done": self.count_of(State::Done),
                "none": self.count_of(State::NoStatus),
            },
            "items": self.entries.iter().map(|entry| serde_json::json!({
                "id": entry.id,
                "title": entry.title,
                "kind": entry.kind,
                "group": entry.group,
                "parent": entry.parent,
                "depends_on": entry.depends_on,
                "state": entry.state.as_key(),
                "lifecycle": if entry.dropped { "dropped" } else { "active" },
                "next_step": entry.next_step,
            })).collect::<Vec<_>>(),
            "sources": self.source_ids(),
        })
    }
}

/// 从当前项目状态构建总览。
#[must_use]
pub fn build(workspace: &Workspace, generated_at: String) -> Overview {
    let config = workspace.config();
    let entries = workspace
        .index()
        .records()
        .iter()
        .filter_map(|record| entry_of(config, record))
        .collect();
    Overview {
        entries,
        groups: config.groups.clone(),
        kinds: config.kinds.iter().map(|kind| kind.name.clone()).collect(),
        generated_at,
    }
}

fn entry_of(config: &crate::workspace::Config, record: &Record) -> Option<Entry> {
    let id = record.id().ok()?.to_string();
    let kind_name = record.kind().unwrap_or("").to_string();
    let state = crate::status::of(config, record);
    Some(Entry {
        id,
        title: record.title().to_string(),
        kind: kind_name,
        group: record.group().to_string(),
        parent: record.parent(),
        depends_on: record.depends_on(),
        state,
        next_step: next_step_of(record),
        dropped: record.lifecycle().is_dropped(),
    })
}

/// 记录的「下一步」：优先取 kind 或记录中的下一步分节，其次取最后一条进展。
fn next_step_of(record: &Record) -> Option<String> {
    for section in ["下一步", "当前下一步"] {
        if let Some(found) = record.body.section(section)
            && !found.body.trim().is_empty()
        {
            return Some(found.body.trim().to_string());
        }
    }
    record
        .body
        .section("进展")
        .and_then(|section| section.body.lines().next_back())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
}

/// 渲染人读总览文本。
#[must_use]
pub fn render_text(language: &str, overview: &Overview) -> String {
    let labels = labels(language);
    let mut out = String::new();
    out.push_str(labels.overview());
    out.push_str(&fill(
        labels.counts_paren(),
        &[labels.generated_at(), &overview.generated_at],
    ));
    let counts: Vec<String> = [
        (labels.todo(), overview.count_of(State::Todo)),
        (labels.in_progress(), overview.count_of(State::InProgress)),
        (labels.blocked(), overview.count_of(State::Blocked)),
        (labels.unverified(), overview.count_of(State::Unverified)),
        (labels.done(), overview.count_of(State::Done)),
    ]
    .iter()
    .map(|(label, count)| fill(labels.line(), &[label, &count.to_string()]))
    .collect();
    out.push_str(&counts.join("  "));
    out.push('\n');

    for state in State::all() {
        let matching: Vec<&Entry> = overview
            .entries
            .iter()
            .filter(|entry| !entry.dropped && entry.state == state)
            .collect();
        if matching.is_empty() {
            continue;
        }
        out.push_str(&fill(
            labels.heading_count(),
            &[state_label(&labels, state), &matching.len().to_string()],
        ));
        for entry in matching {
            out.push_str(&format!(
                "- {} {} · {} · {}\n",
                entry.id, entry.title, entry.kind, entry.group
            ));
            if let Some(step) = &entry.next_step {
                out.push_str(&fill(labels.indented_line(), &[labels.next_step(), step]));
            }
        }
    }

    let dropped: Vec<&Entry> = overview
        .entries
        .iter()
        .filter(|entry| entry.dropped)
        .collect();
    if !dropped.is_empty() {
        out.push_str(&fill(
            labels.heading_count(),
            &[labels.dropped_items(), &dropped.len().to_string()],
        ));
        for entry in dropped {
            out.push_str(&format!("- {} {}\n", entry.id, entry.title));
        }
    }

    out.push_str(&fill(
        labels.source_line(),
        &[
            labels.sources(),
            &overview.source_ids().join(labels.list_separator()),
        ],
    ));
    out
}

/// 交接摘要关注的可继续推进的状态。
fn handoff_states() -> [State; 3] {
    [State::InProgress, State::Blocked, State::Unverified]
}

/// 该状态是否属于交接摘要关注的范围；待办、已完成与无业务状态都不进入。
fn in_handoff(state: State) -> bool {
    handoff_states().contains(&state)
}

/// 渲染交接摘要文本。
///
/// 与默认总览的差别是有实质的：只列进行中、阻塞、完成但未验证三类，每条都给出下一步
/// 与来源记录 ID（相关依据），并在末尾汇总这三类事项的下一步；待办、已完成与无业务
/// 状态记录不进入交接摘要，也不进入下一步汇总。
#[must_use]
pub fn render_handoff(language: &str, overview: &Overview) -> String {
    let labels = labels(language);
    let mut out = String::new();
    out.push_str(labels.handoff_title());
    out.push_str(&fill(
        labels.counts_paren(),
        &[labels.generated_at(), &overview.generated_at],
    ));

    for state in handoff_states() {
        let matching = overview
            .entries
            .iter()
            .filter(|entry| !entry.dropped && entry.state == state)
            .collect::<Vec<_>>();
        out.push_str(&fill(
            labels.heading_count(),
            &[state_label(&labels, state), &matching.len().to_string()],
        ));
        if matching.is_empty() {
            out.push('\n');
            continue;
        }
        for entry in matching {
            out.push_str(&format!(
                "- {} {} · {} · {}\n",
                entry.id, entry.title, entry.kind, entry.group
            ));
            out.push_str(&fill(
                labels.indented_line(),
                &[
                    labels.next_step(),
                    entry.next_step.as_deref().unwrap_or("—"),
                ],
            ));
            out.push_str(&fill(
                labels.indented_line(),
                &[labels.sources(), entry.id.as_str()],
            ));
        }
    }

    out.push_str(&fill(
        labels.heading_count(),
        &[
            labels.next_step(),
            &overview
                .entries
                .iter()
                .filter(|entry| {
                    !entry.dropped && in_handoff(entry.state) && entry.next_step.is_some()
                })
                .count()
                .to_string(),
        ],
    ));
    for entry in overview
        .entries
        .iter()
        .filter(|entry| !entry.dropped && in_handoff(entry.state) && entry.next_step.is_some())
    {
        out.push_str(&fill(
            labels.issue_line(),
            &[entry.id.as_str(), entry.next_step.as_deref().unwrap_or("")],
        ));
    }
    out
}

/// 交接摘要的机器可读形态；键名稳定，不随项目语言变化。
#[must_use]
pub fn handoff_json(overview: &Overview) -> serde_json::Value {
    let section = |state: State| -> Vec<serde_json::Value> {
        overview
            .entries
            .iter()
            .filter(|entry| !entry.dropped && entry.state == state)
            .map(|entry| {
                serde_json::json!({
                    "id": entry.id,
                    "title": entry.title,
                    "kind": entry.kind,
                    "group": entry.group,
                    "state": entry.state.as_key(),
                    "next_step": entry.next_step,
                    "depends_on": entry.depends_on,
                })
            })
            .collect()
    };
    let next_steps: Vec<serde_json::Value> = overview
        .entries
        .iter()
        .filter(|entry| !entry.dropped && in_handoff(entry.state) && entry.next_step.is_some())
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "next_step": entry.next_step,
            })
        })
        .collect();
    serde_json::json!({
        "generated_at": overview.generated_at,
        "in_progress": section(State::InProgress),
        "blocked": section(State::Blocked),
        "done_unverified": section(State::Unverified),
        "next_steps": next_steps,
        "sources": overview.source_ids(),
    })
}

pub(crate) fn state_label(labels: &crate::output::Labels, state: State) -> &'static str {
    match state {
        State::Todo => labels.todo(),
        State::InProgress => labels.in_progress(),
        State::Blocked => labels.blocked(),
        State::Unverified => labels.unverified(),
        State::Done => labels.done(),
        State::NoStatus => labels.no_status(),
    }
}

/// 把已渲染的摘要写成快照；仅显式保存时调用。
pub fn save_snapshot(
    path: &std::path::Path,
    text: &str,
    generated_at: &str,
) -> crate::error::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| crate::error::WorkspaceError::from(error).at(parent))?;
    }
    let mut text = text.to_string();
    text.push_str(&format!("\n<!-- generated_at: {generated_at} -->\n"));
    std::fs::write(path, text)
        .map_err(|error| crate::error::WorkspaceError::from(error).at(path))?;
    Ok(())
}

/// 打印总览：JSON 或人读文本。
pub fn print_overview(
    mode: crate::output::OutputMode,
    language: &str,
    overview: &Overview,
) -> crate::error::Result<()> {
    if mode.is_json() {
        print_json(&overview.to_json())
    } else {
        println!("{}", render_text(language, overview));
        Ok(())
    }
}
