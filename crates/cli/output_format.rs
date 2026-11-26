use jjpwrgem_parse::settings::{choice, choice_enum, settings};

choice_enum! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum OutputFormat default Pretty {
        /// Human readable output
        Pretty = "pretty",
        /// A single line of json describing the result
        Json = "json",
    }
}

settings! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "camelCase", default)]
    struct OutputSettings, table OUTPUT_SETTINGS, id OutputSetting {
        /// Format used to report results and errors
        OutputFormat output_format: OutputFormat = choice().shorthand("json", None, "json"),
    }
}
