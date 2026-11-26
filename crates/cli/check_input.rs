use jjpwrgem_parse::settings::{choice, choice_enum, settings};

choice_enum! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum CheckParser {
        /// A single json document
        #[default]
        Json = "json",
        /// A single json document with comments and trailing commas
        Jsonc = "jsonc",
        /// One json document per line
        Jsonlines = "jsonlines",
    }
}

settings! {
    #[derive(::serde::Deserialize)]
    #[serde(rename_all = "camelCase", default)]
    struct CheckInputSettings, id CheckInputSetting {
        /// Parser to use for input
        Parser parser: CheckParser = choice(),
    }
}
