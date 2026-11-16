use itertools::Itertools;
use jjpwrgem_parse::{
    format::FormatSetting,
    settings::{Setting, SettingKind},
};

use crate::vscode_settings::ignored_conditions;

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
