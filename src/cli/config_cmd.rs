//! `init` 与项目配置定位。

use std::path::{Path, PathBuf};

use crate::error::{Result, WorkspaceError};
use crate::output::labels;
use crate::workspace::config::{Config, DEFAULT_LANGUAGE, DEFAULT_ROOT};

use super::Context;
use super::args::InitArgs;

/// 定位 `worklog.toml`：显式 `--project` 优先，否则从当前目录向上查找。
pub fn find_config(project: Option<&Path>) -> Result<PathBuf> {
    if let Some(project) = project {
        let candidate = project.join(crate::workspace::CONFIG_FILE);
        if !candidate.is_file() {
            return Err(WorkspaceError::usage(format!(
                "no {} in `{}`",
                crate::workspace::CONFIG_FILE,
                project.display()
            ))
            .at(&candidate));
        }
        return Ok(candidate);
    }
    let start = std::env::current_dir().map_err(|error| {
        WorkspaceError::runtime(format!("cannot read current directory: {error}"))
    })?;
    let mut current: Option<&Path> = Some(start.as_path());
    while let Some(directory) = current {
        let candidate = directory.join(crate::workspace::CONFIG_FILE);
        if candidate.is_file() {
            return Ok(candidate);
        }
        current = directory.parent();
    }
    Err(WorkspaceError::usage(format!(
        "no {} found in this directory or its parents; run `worklog init` first",
        crate::workspace::CONFIG_FILE
    )))
}

/// 初始化项目配置与推荐目录；不写入 kind/group 种子数据。
pub fn init(context: &Context, args: &InitArgs) -> Result<()> {
    let project_dir = match context.project.as_deref() {
        Some(project) => project.to_path_buf(),
        None => std::env::current_dir().map_err(|error| {
            WorkspaceError::runtime(format!("cannot read current directory: {error}"))
        })?,
    };
    std::fs::create_dir_all(&project_dir)
        .map_err(|error| WorkspaceError::from(error).at(&project_dir))?;
    let config_path = project_dir.join(crate::workspace::CONFIG_FILE);
    if config_path.exists() && !args.force {
        return Err(WorkspaceError::runtime(format!(
            "{} already exists; pass --force to rewrite it",
            config_path.display()
        ))
        .at(&config_path));
    }

    // 取值顺序：CLI 参数 → 项目配置 → 内置默认值。
    let root = context
        .root
        .as_deref()
        .map(|path| path.display().to_string())
        .or_else(|| existing_value(&config_path, "root"))
        .unwrap_or_else(|| DEFAULT_ROOT.to_string());
    let language = context
        .language
        .clone()
        .or_else(|| existing_value(&config_path, "language"))
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());
    let text = format!("language = \"{language}\"\nroot = \"{root}\"\n");
    std::fs::write(&config_path, text)
        .map_err(|error| WorkspaceError::from(error).at(&config_path))?;

    let config = Config::load(&config_path, None)?;
    for directory in [
        config.items_dir(),
        config.templates_dir(),
        config.summaries_dir(),
    ] {
        std::fs::create_dir_all(&directory)
            .map_err(|error| WorkspaceError::from(error).at(&directory))?;
    }

    println!(
        "{} {}（root = {}）",
        labels(&config.language).created(),
        config_path.display(),
        config.root.display()
    );
    println!(
        "下一步：在 {} 中声明 kind 与 group",
        crate::workspace::CONFIG_FILE
    );
    Ok(())
}

/// 读取已存在配置里的顶层字符串键，供 `init` 沿用取值。
fn existing_value(config_path: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(config_path).ok()?;
    let value: toml::Value = toml::from_str(&text).ok()?;
    value.get(key)?.as_str().map(str::to_string)
}
