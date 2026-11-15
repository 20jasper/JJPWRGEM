use std::fmt;

use super::{
    DEFAULT_PREFERRED_WIDTH, FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions,
};
use crate::settings::{SettingKind, settings};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Parser {
    #[default]
    Json,
    Jsonlines,
}

settings! {
    pub struct FormatSettings, table FORMAT_SETTINGS {
        /// Preferred maximum line width. Note this is not a hard maximum width
        pub preferred_width: Option<usize> = SettingKind::Integer { default: DEFAULT_PREFERRED_WIDTH },
        /// Line ending to use when formatting output
        pub end_of_line: LineEnding = SettingKind::Choice { values: &["lf", "crlf"], default: "lf" },
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormatConfig {
    pub parser: Parser,
    pub uglify: bool,
    pub settings: FormatSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatOption {
    Uglify,
    Jsonlines,
    PreferredWidth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Conflicts(FormatOption, FormatOption),
}

pub const FORMAT_RULES: &[Rule] = &[
    Rule::Conflicts(FormatOption::Uglify, FormatOption::Jsonlines),
    Rule::Conflicts(FormatOption::PreferredWidth, FormatOption::Jsonlines),
    Rule::Conflicts(FormatOption::Uglify, FormatOption::PreferredWidth),
];

impl FormatOption {
    pub fn for_setting(name: &str) -> Option<Self> {
        [Self::PreferredWidth]
            .into_iter()
            .find(|option| option.setting_name() == Some(name))
    }

    pub fn setting_name(self) -> Option<&'static str> {
        match self {
            Self::PreferredWidth => Some("preferred_width"),
            Self::Uglify | Self::Jsonlines => None,
        }
    }

    fn is_active(self, config: &FormatConfig) -> bool {
        match self {
            Self::Uglify => config.uglify,
            Self::Jsonlines => config.parser == Parser::Jsonlines,
            Self::PreferredWidth => config.settings.preferred_width.is_some(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Uglify => "uglify",
            Self::Jsonlines => "jsonlines",
            Self::PreferredWidth => "a preferred width",
        }
    }
}

impl Rule {
    fn is_broken(self, config: &FormatConfig) -> bool {
        match self {
            Self::Conflicts(a, b) => a.is_active(config) && b.is_active(config),
        }
    }

    pub fn describe(self, name: impl Fn(FormatOption) -> String) -> String {
        match self {
            Self::Conflicts(a, b) => format!("{} cannot be combined with {}", name(a), name(b)),
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
    (!others.is_empty()).then(|| format!("cannot be combined with {}", join(&others)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigError(pub Rule);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.describe(|option| option.name().to_owned()))
    }
}

impl std::error::Error for ConfigError {}

impl FormatConfig {
    pub fn validate(self) -> Result<FormatRequest, ConfigError> {
        if let Some(rule) = FORMAT_RULES
            .iter()
            .copied()
            .find(|rule| rule.is_broken(&self))
        {
            return Err(ConfigError(rule));
        }
        let mode = if self.uglify {
            JsonMode::Uglify
        } else {
            JsonMode::Prettify(
                PrettifyOptions::builder()
                    .maybe_preferred_width(self.settings.preferred_width)
                    .line_ending(self.settings.end_of_line)
                    .build(),
            )
        };
        Ok(match self.parser {
            Parser::Json => FormatRequest::Json(mode),
            Parser::Jsonlines => FormatRequest::Jsonlines(
                JsonlinesOptions::builder()
                    .line_ending(self.settings.end_of_line)
                    .build(),
            ),
        })
    }
}
