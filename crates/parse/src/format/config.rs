use super::{
    DEFAULT_PREFERRED_WIDTH, FormatRequest, JsonMode, JsonlinesOptions, LineEnding,
    PrettifyOptions,
    rules::{ConfigError, FORMAT_RULES},
};
use crate::settings::{choice, choice_enum, integer, settings};

choice_enum! {
    pub enum Parser default Json {
        /// A single json document
        Json = "json",
        /// One json document per line
        Jsonlines = "jsonlines",
    }
}

choice_enum! {
    pub enum FormatMode default Pretty {
        /// Pretty print with indentation and line breaks
        Pretty = "pretty",
        /// Remove all insignificant whitespace, also known as minifying
        Uglify = "uglify",
    }
}

settings! {
    struct InputSettings, table INPUT_SETTINGS, id InputSetting {
        /// Parser to use for input
        Parser parser: Parser = choice(),
    }
}

settings! {
    struct FormatSettings, table FORMAT_SETTINGS, id FormatSetting {
        /// Output style
        Mode mode: FormatMode = choice().shorthand("uglify", 'u', "uglify"),
        /// Preferred maximum line width. Note this is not a hard maximum width
        PreferredWidth preferred_width: Option<usize> = integer(DEFAULT_PREFERRED_WIDTH),
        /// Line ending to use when formatting output
        EndOfLine end_of_line: LineEnding = choice().aliases(&["eol"]),
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormatConfig {
    pub input: InputSettings,
    pub settings: FormatSettings,
}

impl FormatConfig {
    fn json_mode(&self) -> JsonMode {
        match self.settings.mode {
            FormatMode::Uglify => JsonMode::Uglify,
            FormatMode::Pretty => JsonMode::Prettify(
                PrettifyOptions::builder()
                    .maybe_preferred_width(self.settings.preferred_width)
                    .line_ending(self.settings.end_of_line)
                    .build(),
            ),
        }
    }
}

impl JsonMode {
    pub fn from_settings(settings: &FormatSettings) -> Self {
        FormatConfig {
            input: InputSettings::default(),
            settings: settings.clone(),
        }
        .drop_ignored()
        .json_mode()
    }
}

impl TryFrom<FormatConfig> for FormatRequest {
    type Error = ConfigError;

    fn try_from(config: FormatConfig) -> Result<Self, ConfigError> {
        if let Some(rule) = FORMAT_RULES
            .iter()
            .copied()
            .find(|rule| rule.is_broken(&config))
        {
            return Err(ConfigError(rule));
        }
        let mode = config.json_mode();
        Ok(match config.input.parser {
            Parser::Json => FormatRequest::Json(mode),
            Parser::Jsonlines => FormatRequest::Jsonlines(
                JsonlinesOptions::builder()
                    .line_ending(config.settings.end_of_line)
                    .build(),
            ),
        })
    }
}
