use itertools::Itertools;
use jjpwrgem_parse::{
    format::FormatSetting,
    settings::{Setting, SettingKind},
};

use crate::vscode_settings::ignored_conditions;

fn ignored_sentences<'a>(
    setting: &'a Setting,
    prefix: &'a str,
) -> impl Iterator<Item = String> + 'a {
    ignored_conditions(setting)
        .map(move |(key, value)| format!("Ignored when `{prefix}{key}` is `\"{value}\"`."))
}

fn section(setting: &Setting, prefix: &str) -> String {
    let details = match setting.kind {
        SettingKind::Integer { default } => vec![format!("Default `{default}`.")],
        SettingKind::Choice {
            values,
            descriptions,
            default,
        } => vec![
            values
                .iter()
                .zip(descriptions)
                .map(|(value, description)| {
                    let marker = if *value == default { " (default)" } else { "" };
                    format!("- `\"{value}\"`{marker}: {description}")
                })
                .join("\n"),
        ],
    };
    let ignored = ignored_sentences(setting, prefix).join(" ");
    [
        format!("### `{prefix}{}`", setting.key()),
        setting.description.to_owned(),
    ]
    .into_iter()
    .chain(details)
    .chain((!ignored.is_empty()).then_some(ignored))
    .join("\n\n")
}

pub fn settings_reference(prefix: &str) -> String {
    let sections = FormatSetting::ALL
        .iter()
        .map(|id| section(id.setting(), prefix))
        .join("\n\n");
    format!("## Settings\n\n{sections}")
}

fn value(setting: &Setting) -> String {
    match setting.kind {
        SettingKind::Integer { default } => default.to_string(),
        SettingKind::Choice { default, .. } => format!("\"{default}\""),
    }
}

fn comments(setting: &Setting) -> Vec<String> {
    let choices = match setting.kind {
        SettingKind::Choice {
            values,
            descriptions,
            ..
        } => values
            .iter()
            .zip(descriptions)
            .map(|(value, description)| format!("  \"{value}\": {description}"))
            .collect(),
        SettingKind::Integer { .. } => Vec::new(),
    };
    std::iter::once(setting.description.to_owned())
        .chain(
            ignored_conditions(setting)
                .map(|(key, value)| format!("Ignored when {key} is \"{value}\"")),
        )
        .chain(choices)
        .collect()
}

pub fn helix_format_config() -> String {
    let settings = FormatSetting::ALL
        .iter()
        .map(|id| {
            let setting = id.setting();
            comments(setting)
                .into_iter()
                .map(|line| format!("# {line}\n"))
                .chain([format!("# {} = {}\n", setting.key(), value(setting))])
                .collect::<String>()
        })
        .join("\n");
    format!("[language-server.jjp.config.format]\n{settings}")
}
