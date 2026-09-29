//! 项目配置：读取 `itemark.toml`，解析 Itemark root、语言、group 与 kind 定义。
//!
//! 取值顺序为显式 CLI 参数 → 项目 `itemark.toml` → 内置默认值。项目配置只记录
//! 事项目录、少量查询默认值和显式语言，不承载工作流规则。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{WorkspaceError, read_error};

pub const CONFIG_FILE: &str = "itemark.toml";
pub const DEFAULT_LANGUAGE: &str = "zh-CN";
pub const DEFAULT_ROOT: &str = "itemark-records";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupConfig {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub required_sections: Vec<String>,
    #[serde(default)]
    pub completion_field: Option<String>,
    #[serde(default)]
    pub completion_values: Vec<String>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

impl KindConfig {
    /// 承载业务状态的字段：优先 kind 里名为 `status` 的声明字段，否则回退到完成字段；
    /// 两者都没有时该 kind 没有业务状态（例如事实、术语类记录）。
    #[must_use]
    pub fn status_field(&self) -> Option<&str> {
        if let Some(field) = self.fields.iter().find(|field| field.name == "status") {
            return Some(field.name.as_str());
        }
        self.completion_field.as_deref()
    }

    /// 声明为完成判定用的字段与完成值；未声明时返回 `None`。
    #[must_use]
    pub fn completion_rule(&self) -> Option<(&str, &[String])> {
        let field = self.completion_field.as_deref()?;
        Some((field, &self.completion_values))
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|field| field.name == name)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub kinds: Vec<KindConfig>,
    #[serde(default)]
    pub groups: Vec<GroupConfig>,
}

impl ConfigFile {
    /// 解析并校验项目配置；键名拼写与声明自洽性都在这里失败。
    pub fn parse(text: &str, path: &Path) -> Result<Self, WorkspaceError> {
        let file: Self = toml::from_str(text).map_err(|error| {
            WorkspaceError::runtime(format!("cannot parse project config: {error}")).at(path)
        })?;
        validate(&file, path)?;
        Ok(file)
    }
}

/// 已解析并做过路径处理的项目配置。
#[derive(Debug, Clone)]
pub struct Config {
    /// `itemark.toml` 所在目录，root 与模板相对它解析。
    pub project_dir: PathBuf,
    /// 配置文件路径；`init` 之外的项目通常必须存在。
    pub config_path: PathBuf,
    pub language: String,
    pub root: PathBuf,
    pub kinds: Vec<KindConfig>,
    pub groups: Vec<String>,
    /// `itemark.toml` 原文；`init` 之外的写入只在需要追加配置时回写它。
    pub raw_text: String,
}

impl Config {
    pub fn load(config_path: &Path, root_override: Option<&Path>) -> Result<Self, WorkspaceError> {
        let config_path = absolute(config_path)?;
        let project_dir = config_path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);

        let (file, raw_text) = if config_path.is_file() {
            let text = std::fs::read_to_string(&config_path)
                .map_err(|error| read_error(&config_path, error))?;
            (ConfigFile::parse(&text, &config_path)?, text)
        } else {
            (
                ConfigFile {
                    language: None,
                    root: None,
                    kinds: Vec::new(),
                    groups: Vec::new(),
                },
                String::new(),
            )
        };

        let language = file
            .language
            .clone()
            .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());
        let root = match root_override {
            Some(path) => absolute_from(&project_dir, path),
            None => {
                let configured = file.root.unwrap_or_else(|| DEFAULT_ROOT.to_string());
                absolute_from(&project_dir, Path::new(&configured))
            }
        };

        Ok(Self {
            project_dir,
            config_path,
            language,
            root,
            kinds: file.kinds,
            groups: file.groups.into_iter().map(|group| group.name).collect(),
            raw_text,
        })
    }

    #[must_use]
    pub fn kind(&self, name: &str) -> Option<&KindConfig> {
        self.kinds.iter().find(|kind| kind.name == name)
    }

    pub fn require_kind(&self, name: &str) -> Result<&KindConfig, WorkspaceError> {
        self.kind(name).ok_or_else(|| {
            let known: Vec<&str> = self.kinds.iter().map(|kind| kind.name.as_str()).collect();
            let known = if known.is_empty() {
                "none declared".to_string()
            } else {
                known.join(", ")
            };
            WorkspaceError::usage(format!("unknown kind `{name}`; declared kinds: {known}"))
        })
    }

    #[must_use]
    pub fn has_group(&self, name: &str) -> bool {
        self.groups.iter().any(|group| group == name)
    }

    #[must_use]
    pub fn items_dir(&self) -> PathBuf {
        self.root.join("items")
    }

    #[must_use]
    pub fn archive_dir(&self) -> PathBuf {
        self.root.join("archive")
    }

    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.root.join("templates")
    }

    #[must_use]
    pub fn summaries_dir(&self) -> PathBuf {
        self.root.join("summaries")
    }

    /// kind 模板路径：相对 Itemark root 解析，与进程当前目录无关。
    #[must_use]
    pub fn template_path(&self, kind: &KindConfig) -> Option<PathBuf> {
        kind.template
            .as_ref()
            .map(|template| absolute_from(&self.root, Path::new(template)))
    }
}

fn validate(file: &ConfigFile, config_path: &Path) -> Result<(), WorkspaceError> {
    let mut kind_names = BTreeSet::new();
    for kind in &file.kinds {
        if kind.name.trim().is_empty() {
            return Err(WorkspaceError::runtime("kind name must not be empty").at(config_path));
        }
        if !kind_names.insert(kind.name.as_str()) {
            return Err(
                WorkspaceError::runtime(format!("duplicate kind name `{}`", kind.name))
                    .at(config_path),
            );
        }
        if kind.completion_field.is_some() && kind.completion_values.is_empty() {
            return Err(WorkspaceError::runtime(format!(
                "kind `{}` declares completion_field without completion_values",
                kind.name
            ))
            .at(config_path));
        }
        let mut field_names = BTreeSet::new();
        for field in &kind.fields {
            if !field_names.insert(field.name.as_str()) {
                return Err(WorkspaceError::runtime(format!(
                    "kind `{}` declares field `{}` twice",
                    kind.name, field.name
                ))
                .at(config_path));
            }
            validate_field(kind, field, config_path)?;
        }
        if let Some(field) = kind.completion_field.as_deref()
            && kind.field(field).is_none()
        {
            return Err(WorkspaceError::runtime(format!(
                "kind `{}` completion_field `{field}` is not a declared field",
                kind.name
            ))
            .at(config_path));
        }
    }

    let mut group_names = BTreeSet::new();
    for group in &file.groups {
        if group.name.trim().is_empty() {
            return Err(WorkspaceError::runtime("group name must not be empty").at(config_path));
        }
        if !group_names.insert(group.name.as_str()) {
            return Err(
                WorkspaceError::runtime(format!("duplicate group name `{}`", group.name))
                    .at(config_path),
            );
        }
    }
    Ok(())
}

fn validate_field(
    kind: &KindConfig,
    field: &FieldDef,
    config_path: &Path,
) -> Result<(), WorkspaceError> {
    let Some(field_type) = crate::kind::FieldType::parse(&field.field_type) else {
        return Err(WorkspaceError::runtime(format!(
            "kind `{}` field `{}` uses unsupported type `{}`; available: {}",
            kind.name,
            field.name,
            field.field_type,
            crate::kind::FieldType::available()
        ))
        .at(config_path));
    };
    if field_type.takes_values() && field.values.is_empty() {
        return Err(WorkspaceError::runtime(format!(
            "kind `{}` enum field `{}` declares no values",
            kind.name, field.name
        ))
        .at(config_path));
    }
    if !field_type.takes_values() && !field.values.is_empty() {
        return Err(WorkspaceError::runtime(format!(
            "kind `{}` field `{}` is not an enum but declares values",
            kind.name, field.name
        ))
        .at(config_path));
    }
    Ok(())
}

/// 把相对路径解析为基于 `base` 的路径，不依赖进程当前目录。
#[must_use]
pub fn absolute_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn absolute(path: &Path) -> Result<PathBuf, WorkspaceError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(WorkspaceError::from)?;
    Ok(cwd.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<ConfigFile, WorkspaceError> {
        ConfigFile::parse(text, Path::new("itemark.toml"))
    }

    #[test]
    fn unknown_keys_in_the_project_config_are_rejected() {
        let error = parse("unknown = 1\n").expect_err("unknown key");
        assert!(error.message().contains("cannot parse"));
    }

    #[test]
    fn completion_field_without_values_is_rejected() {
        let error = parse("[[kinds]]\nname = \"work\"\ncompletion_field = \"status\"\n")
            .expect_err("missing completion_values");
        assert!(error.message().contains("completion_values"));
    }

    #[test]
    fn completion_field_must_be_a_declared_field() {
        let error = parse(
            "[[kinds]]\nname = \"work\"\ncompletion_field = \"status\"\ncompletion_values = [\"done\"]\n",
        )
        .expect_err("undeclared completion field");
        assert!(error.message().contains("completion_field"));
    }

    #[test]
    fn duplicate_names_are_rejected() {
        assert!(parse("[[groups]]\nname = \"研究\"\n[[groups]]\nname = \"研究\"\n").is_err());
        assert!(parse("[[kinds]]\nname = \"work\"\n[[kinds]]\nname = \"work\"\n").is_err());
    }

    #[test]
    fn enum_fields_need_values_and_others_must_not_have_them() {
        assert!(
            parse("[[kinds]]\nname = \"k\"\nfields = [{ name = \"s\", type = \"enum\" }]\n")
                .is_err()
        );
        assert!(
            parse(
                "[[kinds]]\nname = \"k\"\nfields = [{ name = \"s\", type = \"string\", values = [\"a\"] }]\n"
            )
            .is_err()
        );
    }

    #[test]
    fn minimal_config_uses_builtin_defaults() {
        let file = parse("").expect("empty config");
        assert!(file.language.is_none() && file.root.is_none());
        assert!(file.kinds.is_empty() && file.groups.is_empty());
    }
}
