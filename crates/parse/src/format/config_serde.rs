use serde::{Deserialize, Deserializer};

use super::{FormatConfig, LineEnding, Parser};

#[derive(Deserialize)]
#[serde(remote = "LineEnding", rename_all = "lowercase")]
enum LineEndingDef {
    Lf,
    CrLf,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Settings {
    preferred_width: Option<usize>,
    #[serde(with = "LineEndingDef", rename = "endOfLine")]
    line_ending: LineEnding,
}

impl From<Settings> for FormatConfig {
    fn from(
        Settings {
            preferred_width,
            line_ending,
        }: Settings,
    ) -> Self {
        Self {
            parser: Parser::default(),
            uglify: false,
            preferred_width,
            line_ending,
        }
    }
}

impl<'de> Deserialize<'de> for FormatConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Settings::deserialize(deserializer).map(Self::from)
    }
}

impl<'de> Deserialize<'de> for LineEnding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        LineEndingDef::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::format::{FORMAT_SETTINGS, SettingKind};

    fn config(name: &str, value: &Value) -> serde_json::Result<FormatConfig> {
        serde_json::from_value(json!({ name: value }))
    }

    #[test]
    fn every_setting_names_a_field_and_its_values_deserialize() {
        let initial = FormatConfig::default();
        for setting in FORMAT_SETTINGS {
            match setting.kind {
                SettingKind::Integer { default } => {
                    assert!(
                        config(setting.name, &json!(default)).is_ok(),
                        "{}",
                        setting.name
                    );
                    assert_ne!(
                        config(setting.name, &json!(default + 1)).ok(),
                        Some(initial.clone()),
                        "{}",
                        setting.name,
                    );
                }
                SettingKind::Choice { values, default } => {
                    for value in values {
                        let parsed = config(setting.name, &json!(value)).ok();
                        assert!(parsed.is_some(), "{} = {value}", setting.name);
                        assert_eq!(
                            parsed == Some(initial.clone()),
                            *value == default,
                            "{} = {value}",
                            setting.name,
                        );
                    }
                }
            }
        }
    }
}
