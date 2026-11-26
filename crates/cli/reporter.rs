use jjpwrgem_parse::settings::{choice, choice_enum, settings};
use jjpwrgem_ui::{Color, Style};

choice_enum! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Reporter {
        /// Human readable output
        #[default]
        Pretty = "pretty",
        /// A single line of json describing the result
        Json = "json",
    }
}

impl From<Reporter> for Style {
    fn from(format: Reporter) -> Self {
        match format {
            Reporter::Pretty => Self::Pretty(Color::Plain),
            Reporter::Json => Self::Json,
        }
    }
}

settings! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "camelCase", default)]
    struct ReporterSettings, id ReporterSetting {
        /// Format used to report results and errors
        Reporter reporter: Reporter = choice().shorthand("json", None, "json"),
    }
}
