use std::fs;

use anyhow::{Context, Result, bail};
use jjpwrgem_parse::{
    format::FORMAT_SETTINGS,
    settings::{Setting, SettingKind},
};
use serde_json::{Map, Value, json};

const PACKAGE_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../npm-packages/jjpwrgem-vscode/package.json"
);
const SETTINGS_PREFIX: &str = "jjpwrgem.format.";
const PROPERTIES_POINTER: &str = "/contributes/configuration/properties";

fn property(setting: &Setting) -> Value {
    match setting.kind {
        SettingKind::Flag => json!({
            "type": "boolean",
            "default": false,
            "description": setting.description,
        }),
        SettingKind::Integer { default } => json!({
            "type": ["integer", "null"],
            "minimum": 0,
            "default": null,
            "description": format!("{}. Defaults to {default} when unset", setting.description),
        }),
        SettingKind::Choice { values, default } => json!({
            "type": "string",
            "enum": values,
            "default": default,
            "description": setting.description,
        }),
    }
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
    *properties = kept
        .into_iter()
        .chain(FORMAT_SETTINGS.iter().map(|setting| {
            (
                format!("{SETTINGS_PREFIX}{}", setting.key()),
                property(setting),
            )
        }))
        .collect();
    Ok(serde_json::to_string_pretty(&package)? + "\n")
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
