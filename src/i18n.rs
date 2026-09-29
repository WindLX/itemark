//! 项目语言选择与静态 CLI 文案词典。

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

static ZH: OnceLock<HashMap<String, String>> = OnceLock::new();
static EN: OnceLock<HashMap<String, String>> = OnceLock::new();

fn dictionary(language: &str) -> &'static HashMap<String, String> {
    let (slot, source) = if is_english(language) {
        (&EN, include_str!("../i18n/en.json"))
    } else {
        (&ZH, include_str!("../i18n/zh-CN.json"))
    };
    slot.get_or_init(|| serde_json::from_str(source).expect("bundled locale JSON is valid"))
}

#[must_use]
pub fn is_english(language: &str) -> bool {
    language.eq_ignore_ascii_case("en") || language.to_ascii_lowercase().starts_with("en-")
}

#[must_use]
pub fn text(key: &str, language: &str) -> &'static str {
    optional_text(key, language).unwrap_or_else(|| panic!("missing translation key `{key}`"))
}

#[must_use]
pub fn optional_text(key: &str, language: &str) -> Option<&'static str> {
    dictionary(language).get(key).map(String::as_str)
}

#[must_use]
pub fn argument_help(key: &str, language: &str) -> Option<&'static str> {
    optional_text(&format!("arg_{key}"), language)
}

/// Normalize the active system locale to one of the shipped catalogs.
#[must_use]
pub fn system_language() -> &'static str {
    match sys_locale::get_locale().as_deref() {
        Some(locale) if locale.to_ascii_lowercase().starts_with("en") => "en",
        _ => "zh-CN",
    }
}

/// Select help/error language before Clap parses `--help` or rejects an argument.
/// `init` always uses the active system locale; other commands prefer project config.
#[must_use]
pub fn language_for_args(args: &[OsString]) -> &'static str {
    let project = project_argument(args).unwrap_or_else(discover_project);
    if first_command(args).as_deref() == Some("init") {
        return system_language();
    }
    let config = project.join(crate::workspace::CONFIG_FILE);
    std::fs::read_to_string(config)
        .ok()
        .and_then(|source| toml::from_str::<toml::Value>(&source).ok())
        .and_then(|value| value.get("language")?.as_str().map(str::to_owned))
        .map_or_else(system_language, |language| {
            if is_english(&language) { "en" } else { "zh-CN" }
        })
}

fn project_argument(args: &[OsString]) -> Option<PathBuf> {
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_string_lossy();
        if arg == "--project" {
            return args.get(index + 1).map(PathBuf::from);
        }
        if let Some(path) = arg.strip_prefix("--project=") {
            return Some(PathBuf::from(path));
        }
        index += 1;
    }
    None
}

fn discover_project() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut current = Some(start.as_path());
    while let Some(directory) = current {
        if directory.join(crate::workspace::CONFIG_FILE).is_file() {
            return directory.to_path_buf();
        }
        current = directory.parent();
    }
    start
}

fn first_command(args: &[OsString]) -> Option<String> {
    let mut skip_next = false;
    for arg in args {
        let value = arg.to_string_lossy();
        if skip_next {
            skip_next = false;
            continue;
        }
        if value == "--project" || value == "--root" || value == "--color" {
            skip_next = true;
            continue;
        }
        if value.starts_with('-') || value.contains('=') {
            continue;
        }
        return Some(value.into_owned());
    }
    None
}

/// Translate the stable English diagnostic fragments used by the CLI.
#[must_use]
pub fn localize_diagnostic(message: &str, language: &str) -> String {
    if is_english(language) {
        return message.to_string();
    }
    let replacements = [
        (
            "no itemark.toml found in this directory or its parents; run `itemark init` first",
            "diag_no_config",
        ),
        ("no itemark.toml in", "diag_no_config_at"),
        ("required field", "diag_required_field"),
        ("is missing or empty", "diag_section_missing"),
        (
            "completion value is recorded but is missing",
            "diag_completion_value",
        ),
        ("is missing", "diag_field_missing"),
        ("is empty", "diag_field_empty"),
        ("required section", "diag_required_section"),
        ("record has no", "diag_record_no"),
        (
            "setting the completion field to its completion value requires a non-empty ",
            "diag_completion_required",
        ),
        (" and ", "diag_conjunction"),
        (
            "; add it (or mark the result as unverified) before recording completion",
            "diag_completion_before",
        ),
        ("completion_note", "diag_completion_note"),
        ("completion_evidence", "diag_completion_evidence"),
        ("completion note", "diag_completion_note"),
        ("completion evidence", "diag_completion_evidence"),
        ("takes a single record ID", "diag_parent_single"),
        ("use `depends_on` for several records", "diag_depends_many"),
        (
            "expected one of auto, always, never, got",
            "diag_expected_color",
        ),
        ("expected `key=value`, got", "diag_expected_pair"),
        ("a value is required for", "diag_value_required"),
        ("but none was supplied", "diag_none_supplied"),
        ("expected a non-empty key in", "diag_empty_key"),
        ("expected `--date` as YYYY-MM-DD, got", "diag_date_format"),
        (
            "must not reference the record itself",
            "diag_self_reference",
        ),
        ("references unknown record", "diag_unknown_record"),
        ("unknown Itemark item", "diag_unknown_item"),
        ("unknown group", "diag_unknown_group"),
        ("declared groups", "diag_declared_groups"),
        ("unknown kind", "diag_unknown_kind"),
        ("declared kinds", "diag_declared_kinds"),
        ("is not declared in the project config", "diag_not_declared"),
        ("uses unsupported type", "diag_unsupported_type"),
        ("declares no values", "diag_declares_no"),
        ("is not an enum but declares values", "diag_non_enum"),
        ("Itemark item", "diag_itemark_item"),
        ("already exists", "diag_already_exists"),
        ("file already exists", "diag_file_exists"),
        ("cannot read", "diag_cannot_read"),
        ("cannot parse", "diag_cannot_parse"),
        ("changed outside this operation", "diag_changed"),
        ("re-read it and retry", "diag_retry"),
        ("template is missing:", "diag_kind_template_missing"),
        (
            "Itemark root is not initialised; cannot write",
            "diag_root_uninitialized",
        ),
        ("a similar argument exists:", "diag_similar"),
        ("For more information, try", "diag_more"),
        ("itemark check", "diag_itemark_check"),
        ("issue(s)", "diag_issues_count"),
        ("unexpected argument", "diag_unexpected"),
        ("Usage:", "diag_usage"),
        ("Options:", "diag_options"),
        ("Arguments:", "diag_arguments"),
        ("Commands:", "diag_commands"),
        ("error:", "diag_error"),
        ("tip:", "diag_tip"),
    ];
    let mut translated = message.to_string();
    for (from, key) in replacements {
        translated = translated.replace(from, text(key, language));
    }
    translated.replace("found", "")
}
