use jjpwrgem_parse::settings::{choice, choice_enum, settings};
use jjpwrgem_ui::{Color, Style};

choice_enum! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum OutputFormat {
        /// Human readable output
        #[default]
        Pretty = "pretty",
        /// A single line of json describing the result
        Json = "json",
    }
}

impl From<OutputFormat> for Style {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Pretty => Self::Pretty(Color::Plain),
            OutputFormat::Json => Self::Json,
        }
    }
}

settings! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "camelCase", default)]
    struct OutputSettings, id OutputSetting {
        /// Format used to report results and errors
        OutputFormat output_format: OutputFormat = choice().shorthand("json", None, "json"),
    }
}
