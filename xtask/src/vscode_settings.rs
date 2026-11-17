use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use itertools::Itertools;
use jjpwrgem_parse::{
    format::{DEFAULT_PREFERRED_WIDTH, FormatOption, FormatSetting, LineEnding, prettify_str},
    settings::{Setting, SettingKind},
};
use serde_json::{Map, Value, json};

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

fn property(id: FormatSetting) -> Value {
    let setting = id.setting();
    let mut schema = match setting.kind {
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
    };
    schema["markdownDescription"] = markdown_description(setting).into();
    schema["order"] = (id as usize).into();
    schema
}

fn generated_properties() -> Map<String, Value> {
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

fn render(package_json: &str) -> Result<String> {
    let mut package: Value = serde_json::from_str(package_json)?;
    let properties = package
        .pointer_mut(PROPERTIES_POINTER)
        .and_then(Value::as_object_mut)
        .context("package.json has no contributes.configuration.properties")?;
    let kept: Map<String, Value> = std::mem::take(properties)
        .into_iter()
        .filter(|(key, _)| !key.starts_with(SETTINGS_PREFIX))
        .collect();
    *properties = kept.into_iter().chain(generated_properties()).collect();
    let compact = jjpwrgem_parse::format::serde::uglify_serializable(&package)
        .map_err(|error| anyhow!("{error}"))?;
    let pretty = prettify_str(&compact, DEFAULT_PREFERRED_WIDTH, LineEnding::default())
        .map_err(|error| anyhow!("{error}"))?;
    Ok(pretty + "\n")
}

pub fn write_vscode_settings() -> Result<()> {
    fs::write(PACKAGE_JSON, render(&fs::read_to_string(PACKAGE_JSON)?)?)?;
    println!("generated vscode settings");
    Ok(())
}

pub fn are_vscode_settings_updated() -> Result<()> {
    let current = fs::read_to_string(PACKAGE_JSON)?;
    if render(&current)? != current {
        bail!("vscode settings are out of date, run `cargo xtask generate-vscode-settings`");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties_snapshot() {
        insta::assert_json_snapshot!(generated_properties());
    }
}
