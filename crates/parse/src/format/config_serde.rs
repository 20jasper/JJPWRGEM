use serde::{Deserialize, Deserializer};

use super::LineEnding;

#[derive(Deserialize)]
#[serde(remote = "LineEnding", rename_all = "lowercase")]
enum LineEndingDef {
    Lf,
    CrLf,
}

impl<'de> Deserialize<'de> for LineEnding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        LineEndingDef::deserialize(deserializer)
    }
}
