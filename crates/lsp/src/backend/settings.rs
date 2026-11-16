use jjpwrgem_parse::format::FormatSettings;
use serde::Deserialize;
use serde_json::Value;

const SECTION: &str = "jjpwrgem";

#[derive(Deserialize, Default)]
struct Settings {
    #[serde(default)]
    format: FormatSettings,
}

pub fn format_settings(value: Value) -> Result<FormatSettings, serde_json::Error> {
    let value = match value {
        Value::Object(mut map) => map.remove(SECTION).unwrap_or(Value::Object(map)),
        other => other,
    };
    serde_json::from_value::<Option<Settings>>(value)
        .map(|settings| settings.unwrap_or_default().format)
}
