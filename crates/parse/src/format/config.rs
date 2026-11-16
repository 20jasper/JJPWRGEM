use std::fmt;

use FormatOption::{Jsonlines, Uglify};
use FormatSetting::{Mode, PreferredWidth};

use super::{
    DEFAULT_PREFERRED_WIDTH, FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions,
};
use crate::settings::{choice, integer, settings};

choice! {
    pub enum Parser default Json {
        /// A single json document
        Json = "json",
        /// One json document per line
        Jsonlines = "jsonlines",
    }
}

choice! {
    pub enum FormatMode default Pretty {
        /// Pretty print with indentation and line breaks
        Pretty = "pretty",
        /// Remove all insignificant whitespace
        Uglify = "uglify",
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
        /// Pretty print, or remove all insignificant whitespace (uglify, also known as minify)
        Mode mode: FormatMode = choice::<FormatMode>().shorthand("uglify", 'u', "uglify"),
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
    Uglify,
    Setting(FormatSetting),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    IgnoredWhen(FormatSetting, FormatOption),
}

pub const FORMAT_RULES: &[Rule] = &[
    Rule::IgnoredWhen(Mode, Jsonlines),
    Rule::IgnoredWhen(PreferredWidth, Jsonlines),
    Rule::IgnoredWhen(PreferredWidth, Uglify),
];

impl FormatOption {
    fn is_active(self, config: &FormatConfig) -> bool {
        match self {
            Self::Jsonlines => config.input.parser == Parser::Jsonlines,
            Self::Uglify => config.settings.mode == FormatMode::Uglify,
            Self::Setting(id) => config.settings.is_set(id),
        }
    }

    pub fn name(self) -> String {
        match self {
            Self::Jsonlines => "jsonlines".to_owned(),
            Self::Uglify => "uglify".to_owned(),
            Self::Setting(id) => id.setting().name.replace('_', " "),
        }
    }

    pub fn condition(self) -> Option<(FormatSetting, &'static str)> {
        match self {
            Self::Uglify => Some((Mode, FormatMode::Uglify.name())),
            Self::Jsonlines | Self::Setting(_) => None,
        }
    }
}

fn cannot_be_combined_with(others: &str) -> String {
    format!("cannot be combined with {others}")
}

impl Rule {
    fn is_broken(self, config: &FormatConfig) -> bool {
        match self {
            Self::IgnoredWhen(setting, condition) => {
                config.settings.is_set(setting) && condition.is_active(config)
            }
        }
    }

    pub fn describe(self, name: impl Fn(FormatOption) -> String) -> String {
        match self {
            Self::IgnoredWhen(setting, condition) => format!(
                "{} {}",
                name(FormatOption::Setting(setting)),
                cannot_be_combined_with(&name(condition))
            ),
        }
    }
}

fn join(items: &[String]) -> String {
    match items {
        [rest @ .., tail] if !rest.is_empty() => format!("{} or {tail}", rest.join(", ")),
        _ => items.join(""),
    }
}

pub fn rule_clause(
    options: &[FormatOption],
    name: impl Fn(FormatOption) -> String,
) -> Option<String> {
    let others: Vec<String> = FORMAT_RULES
        .iter()
        .filter_map(|rule| match *rule {
            Rule::IgnoredWhen(setting, condition)
                if options.contains(&FormatOption::Setting(setting)) =>
            {
                Some(condition)
            }
            Rule::IgnoredWhen(setting, condition) if options.contains(&condition) => {
                Some(FormatOption::Setting(setting))
            }
            Rule::IgnoredWhen(..) => None,
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

impl FormatConfig {
    #[must_use]
    pub fn drop_ignored(self) -> Self {
        FORMAT_RULES.iter().fold(self, |mut config, rule| {
            let Rule::IgnoredWhen(setting, condition) = *rule;
            if condition.is_active(&config) {
                config.settings.reset(setting);
            }
            config
        })
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
        Ok(Self::from_valid(&config))
    }
}

impl FormatRequest {
    pub fn from_lenient(config: FormatConfig) -> Self {
        Self::from_valid(&config.drop_ignored())
    }

    fn from_valid(config: &FormatConfig) -> Self {
        let mode = match config.settings.mode {
            FormatMode::Uglify => JsonMode::Uglify,
            FormatMode::Pretty => JsonMode::Prettify(
                PrettifyOptions::builder()
                    .maybe_preferred_width(config.settings.preferred_width)
                    .line_ending(config.settings.end_of_line)
                    .build(),
            ),
        };
        match config.input.parser {
            Parser::Json => FormatRequest::Json(mode),
            Parser::Jsonlines => FormatRequest::Jsonlines(
                JsonlinesOptions::builder()
                    .line_ending(config.settings.end_of_line)
                    .build(),
            ),
        }
    }
}
