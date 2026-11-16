use itertools::Itertools;
use jjpwrgem_parse::{
    format::{FORMAT_SETTINGS, FormatSetting},
    settings::{Setting, SettingKind},
};

use crate::vscode_settings::ignored_conditions;

fn value(setting: &Setting) -> String {
    match setting.kind {
        SettingKind::Flag => "false".to_owned(),
        SettingKind::Integer { default } => default.to_string(),
        SettingKind::Choice { default, .. } => format!("\"{default}\""),
    }
}

fn comments(id: FormatSetting) -> Vec<String> {
    let setting = id.setting();
    let choices = match setting.kind {
        SettingKind::Choice { values, .. } => Some(format!(
            "One of {}",
            values.iter().map(|value| format!("\"{value}\"")).join(", ")
        )),
        SettingKind::Flag | SettingKind::Integer { .. } => None,
    };
    std::iter::once(setting.description.to_owned())
        .chain(choices)
        .chain(
            ignored_conditions(id).into_iter().map(|(other, value)| {
                format!("Ignored when {} is \"{value}\"", other.setting().key())
            }),
        )
        .collect()
}

pub fn helix_config() -> String {
    let settings = FormatSetting::ALL
        .iter()
        .zip(FORMAT_SETTINGS)
        .map(|(&id, setting)| {
            comments(id)
                .into_iter()
                .map(|line| format!("# {line}\n"))
                .chain([format!("{} = {}\n", setting.key(), value(setting))])
                .collect::<String>()
        })
        .join("");
    format!(
        "# languages.toml\n[language-server.jjp]\ncommand = \"jjp\"\nargs = [\"lsp\"]\n\n[language-server.jjp.config.format]\n{settings}\n[[language]]\nname = \"json\"\nlanguage-servers = [\"jjp\"]"
    )
}
