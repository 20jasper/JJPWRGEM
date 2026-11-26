use std::fs;

use anyhow::{Context, Result};
use itertools::Itertools;
use jjpwrgem_parse::{
    format::{FormatOption, FormatSetting},
    settings::{Setting, SettingKind},
};
use serde_json::{Map, Value, json};

use crate::{
    generated::{pretty_json, write_or_check},
    vscode_extension::extension_properties,
};

const PACKAGE_JSON: &str = repo_path!("npm-packages/jjpwrgem-vscode/package.json");
const SETTINGS_PREFIX: &str = "jjpwrgem.format.";
const PROPERTIES_POINTER: &str = "/contributes/configuration/properties";

pub fn ignored_conditions(row: &Setting) -> impl Iterator<Item = (String, &'static str)> {
    FormatOption::ignored_when(row)
        .filter(|option| !matches!(option, FormatOption::Parser(_)))
        .filter_map(|option| Some((option.setting().key(), option.value()?)))
}

fn markdown_description(setting: &Setting) -> String {
    std::iter::once(format!("{}.", setting.description))
        .chain(
            ignored_conditions(setting).map(|(key, value)| {
                format!("Ignored when `#{SETTINGS_PREFIX}{key}#` is `{value}`.")
            }),
        )
        .join(" ")
}

pub fn schema(setting: &Setting) -> Value {
    match setting.kind {
        SettingKind::String { default } => json!({
            "type": "string",
            "default": default,
        }),
        SettingKind::Integer { default } => json!({
            "type": "integer",
            "minimum": 0,
            "default": default,
        }),
        SettingKind::Choice {
            values,
            descriptions,
            default,
        } => json!({
            "type": "string",
            "enum": values,
            "enumDescriptions": descriptions,
            "default": default,
        }),
    }
}

fn property(id: FormatSetting) -> Value {
    let setting = id.setting();
    let mut schema = schema(setting);
    schema["markdownDescription"] = markdown_description(setting).into();
    schema["order"] = (id as usize).into();
    schema
}

fn format_properties() -> Map<String, Value> {
    FormatSetting::ALL
        .iter()
        .map(|&id| {
            (
                format!("{SETTINGS_PREFIX}{}", id.setting().key()),
                property(id),
            )
        })
        .collect()
}

fn generated_properties() -> Map<String, Value> {
    let (extension, trace) = extension_properties();
    extension
        .into_iter()
        .chain(format_properties())
        .chain(trace)
        .collect()
}

fn render(package_json: &str) -> Result<String> {
    let mut package: Value = serde_json::from_str(package_json)?;
    let properties = package
        .pointer_mut(PROPERTIES_POINTER)
        .and_then(Value::as_object_mut)
        .context("package.json has no contributes.configuration.properties")?;
    let generated = generated_properties();
    let kept: Map<String, Value> = std::mem::take(properties)
        .into_iter()
        .filter(|(key, _)| !generated.contains_key(key) && !key.starts_with(SETTINGS_PREFIX))
        .collect();
    *properties = kept.into_iter().chain(generated).collect();
    pretty_json(&package)
}

pub fn vscode_settings(check: bool) -> Result<()> {
    let generated = render(&fs::read_to_string(PACKAGE_JSON)?)?;
    write_or_check(PACKAGE_JSON, &generated, "vscode-settings", check)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_snapshot() {
        insta::assert_json_snapshot!(generated_properties());
    }
}
