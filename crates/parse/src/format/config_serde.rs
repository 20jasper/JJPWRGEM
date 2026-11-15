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
