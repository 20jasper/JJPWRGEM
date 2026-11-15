use std::fmt;

use FormatOption::{Jsonlines, Setting};
use FormatSetting::{PreferredWidth, Uglify};

use super::{
    DEFAULT_PREFERRED_WIDTH, FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions,
};
use crate::settings::{choice, flag, integer, settings};

choice! {
    pub enum Parser default Json {
        Json = "json",
        Jsonlines = "jsonlines",
    }
}

settings! {
    struct InputSettings, table INPUT_SETTINGS, id InputSetting {
        /// Parser to use for input
        Parser parser: Parser = choice::<Parser>(),
    }
}

settings! {
    struct FormatSettings, table FORMAT_SETTINGS, id FormatSetting {
        /// Removes all insignificant whitespace instead of pretty printing, also known as minifying
        Uglify uglify: bool = flag().short('u'),
        /// Preferred maximum line width. Note this is not a hard maximum width
        PreferredWidth preferred_width: Option<usize> = integer(DEFAULT_PREFERRED_WIDTH),
        /// Line ending to use when formatting output
        EndOfLine end_of_line: LineEnding = choice::<LineEnding>().aliases(&["eol"]),
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormatConfig {
    pub input: InputSettings,
    pub settings: FormatSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatOption {
    Jsonlines,
    Setting(FormatSetting),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Conflicts(FormatOption, FormatOption),
}

pub const FORMAT_RULES: &[Rule] = &[
    Rule::Conflicts(Setting(Uglify), Jsonlines),
    Rule::Conflicts(Setting(PreferredWidth), Jsonlines),
    Rule::Conflicts(Setting(Uglify), Setting(PreferredWidth)),
];

impl FormatOption {
    fn is_active(self, config: &FormatConfig) -> bool {
        match self {
            Self::Jsonlines => config.input.parser == Parser::Jsonlines,
            Self::Setting(id) => config.settings.is_set(id),
        }
    }

    pub fn name(self) -> String {
        match self {
            Self::Jsonlines => "jsonlines".to_owned(),
            Self::Setting(id) => id.setting().name.replace('_', " "),
        }
    }
}

fn cannot_be_combined_with(others: &str) -> String {
    format!("cannot be combined with {others}")
}

impl Rule {
    fn is_broken(self, config: &FormatConfig) -> bool {
        match self {
            Self::Conflicts(a, b) => a.is_active(config) && b.is_active(config),
        }
    }

    pub fn describe(self, name: impl Fn(FormatOption) -> String) -> String {
        match self {
            Self::Conflicts(a, b) => format!("{} {}", name(a), cannot_be_combined_with(&name(b))),
        }
    }
}

fn join(items: &[String]) -> String {
    match items {
        [rest @ .., tail] if !rest.is_empty() => format!("{} or {tail}", rest.join(", ")),
        _ => items.join(""),
    }
}

pub fn rule_clause(option: FormatOption, name: impl Fn(FormatOption) -> String) -> Option<String> {
    let others: Vec<String> = FORMAT_RULES
        .iter()
        .filter_map(|rule| match *rule {
            Rule::Conflicts(a, b) if a == option => Some(b),
            Rule::Conflicts(a, b) if b == option => Some(a),
            Rule::Conflicts(..) => None,
        })
        .map(name)
        .collect();
    (!others.is_empty()).then(|| cannot_be_combined_with(&join(&others)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigError(pub Rule);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.describe(|option| option.name()))
    }
}

impl std::error::Error for ConfigError {}

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
        let mode = if config.settings.uglify {
            JsonMode::Uglify
        } else {
            JsonMode::Prettify(
                PrettifyOptions::builder()
                    .maybe_preferred_width(config.settings.preferred_width)
                    .line_ending(config.settings.end_of_line)
                    .build(),
            )
        };
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
