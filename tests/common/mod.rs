//! 集成测试共用的临时项目与 CLI 调用辅助。
//!
//! 测试 seam 是真实的 CLI 进程边界：所有断言只依赖命令输出、退出码与磁盘上的
//! 记录文件，不绑定内部函数结构。

#![allow(dead_code, unused_imports)]

// 再导出，让每个测试文件只需 `use common::*;`。
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use std::process::{Command, Output, Stdio};

pub use tempfile::TempDir;

/// 一个临时 Itemark 项目，测试结束后自动清理。
pub struct Project {
    dir: TempDir,
}

impl Project {
    pub fn new() -> Self {
        Self {
            dir: TempDir::new().expect("create temporary project"),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write(&self, relative: &str, contents: &str) {
        let path = self.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(&path, contents).expect("write project file");
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path().join(relative)).expect("read project file")
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.path().join(relative).exists()
    }

    /// Find the current filename by its stable YAML ID, even when the title changes the path.
    pub fn item_file(&self, id: &str) -> PathBuf {
        for entry in fs::read_dir(self.path().join("itemark-records/items")).expect("read items") {
            let path = entry.expect("item entry").path();
            let Ok(record) = itemark::record::Record::read(&path) else {
                continue;
            };
            if record.id().ok() == Some(id) {
                return path;
            }
        }
        panic!("record {id} does not exist");
    }

    /// 写入一个带自定义 kind 的完整项目。
    pub fn configure(&self) {
        self.write("itemark.toml", CONFIG);
        self.write(
            "itemark-records/templates/project-note.md",
            "---\nid: \"{{id}}\"\nkind: \"project-note\"\ngroup: \"{{group}}\"\ntitle: \"{{title}}\"\nphase: \"{{phase}}\"\n---\n## 目标\n\n",
        );
        self.write(
            "itemark-records/templates/work.md",
            "---\nid: \"{{id}}\"\nkind: \"work\"\ngroup: \"{{group}}\"\ntitle: \"{{title}}\"\nstatus: \"{{status}}\"\n---\n## 目标\n\n## 进展\n\n",
        );
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_itemark"));
        command.current_dir(self.path());
        command
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().expect("run Itemark CLI")
    }

    /// 运行并断言成功，返回标准输出。
    pub fn ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "`{}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("stdout is UTF-8")
    }

    /// 运行并断言失败，返回退出码与合并后的输出。
    pub fn fail(&self, args: &[&str]) -> (i32, String) {
        let output = self.run(args);
        let code = output.status.code().expect("process exited with a code");
        assert_ne!(code, 0, "`{}` unexpectedly succeeded", args.join(" "));
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        (code, text)
    }

    /// 添加一条 `project-note` 记录并返回其稳定 ID。
    pub fn add_note(&self, title: &str, group: &str) -> String {
        let output = self.ok(&[
            "add",
            "--kind",
            "project-note",
            "--group",
            group,
            "--set",
            &format!("title={title}"),
            "--set",
            "phase=doing",
            "--json",
        ]);
        ids_in(&output).first().cloned().expect("add returns an ID")
    }

    /// 添加一条 `work` 记录并返回其稳定 ID。
    pub fn add_work(&self, title: &str, status: &str) -> String {
        let output = self.ok(&[
            "add",
            "--kind",
            "work",
            "--group",
            "产品",
            "--set",
            &format!("title={title}"),
            "--set",
            &format!("status={status}"),
            "--json",
        ]);
        ids_in(&output).first().cloned().expect("add returns an ID")
    }
}

pub const CONFIG: &str = r#"
language = "zh-CN"
root = "itemark-records"

[[groups]]
name = "研究"

[[groups]]
name = "产品"

[[kinds]]
name = "project-note"
description = "A user-defined kind"
template = "templates/project-note.md"
required_sections = ["目标"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "phase", type = "enum", required = true, values = ["doing", "done"] },
]

[[kinds]]
name = "work"
description = "可继续推进的工作事项"
template = "templates/work.md"
required_sections = ["目标"]
completion_field = "status"
completion_values = ["done"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "status", type = "enum", required = true, values = ["todo", "in_progress", "blocked", "done"] },
]
"#;

/// 从任意输出中取出稳定 ID，用于断言命令返回的记录标识。
pub fn ids_in(text: &str) -> Vec<String> {
    let mut ids: Vec<String> = text
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .filter(|token| {
            let digits = token
                .strip_prefix("IM-")
                .or_else(|| token.strip_prefix("WL-"));
            digits.is_some_and(|digits| {
                !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit())
            })
        })
        .map(str::to_string)
        .collect();
    ids.dedup();
    ids
}
