use std::fmt;

use super::{
    DEFAULT_PREFERRED_WIDTH, FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions,
};
use crate::settings::{Setting, choice, integer, settings};

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
        Parser parser: Parser = choice(),
    }
}

settings! {
    struct FormatSettings, table FORMAT_SETTINGS, id FormatSetting {
        /// Pretty print, or remove all insignificant whitespace (uglify, also known as minify)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatOption {
    Setting(FormatSetting),
    Mode(FormatMode),
    Parser(Parser),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub option: FormatOption,
    pub ignored_when: FormatOption,
}

pub const FORMAT_RULES: &[Rule] = &[
    Rule {
        option: FormatOption::Mode(FormatMode::Uglify),
        ignored_when: FormatOption::Parser(Parser::Jsonlines),
    },
    Rule {
        option: FormatOption::Setting(FormatSetting::PreferredWidth),
        ignored_when: FormatOption::Parser(Parser::Jsonlines),
    },
    Rule {
        option: FormatOption::Setting(FormatSetting::PreferredWidth),
        ignored_when: FormatOption::Mode(FormatMode::Uglify),
    },
];

impl FormatOption {
    fn is_active(self, config: &FormatConfig) -> bool {
        match self {
            Self::Setting(id) => config.settings.is_set(id),
            Self::Mode(mode) => config.settings.mode == mode,
            Self::Parser(parser) => config.input.parser == parser,
        }
    }

    pub fn setting(self) -> &'static Setting {
        match self {
            Self::Setting(id) => id.setting(),
            Self::Mode(_) => FormatSetting::Mode.setting(),
            Self::Parser(_) => InputSetting::Parser.setting(),
        }
    }

    pub fn value(self) -> Option<&'static str> {
        match self {
            Self::Setting(_) => None,
            Self::Mode(mode) => Some(mode.name()),
            Self::Parser(parser) => Some(parser.name()),
        }
    }

    pub fn name(self) -> String {
        let setting = self.setting().name.replace('_', " ");
        self.value()
            .map_or_else(|| setting.clone(), |value| format!("{setting} {value}"))
    }

    fn format_setting(self) -> Option<FormatSetting> {
        match self {
            Self::Setting(id) => Some(id),
            Self::Mode(_) => Some(FormatSetting::Mode),
            Self::Parser(_) => None,
        }
    }

    pub fn for_setting(row: &Setting) -> Vec<Self> {
        FORMAT_RULES
            .iter()
            .flat_map(|rule| [rule.option, rule.ignored_when])
            .filter(|option| option.setting().name == row.name)
            .fold(Vec::new(), |mut options, option| {
                if !options.contains(&option) {
                    options.push(option);
                }
                options
            })
    }
}

fn cannot_be_combined_with(others: &str) -> String {
    format!("cannot be combined with {others}")
}

impl Rule {
    fn is_broken(self, config: &FormatConfig) -> bool {
        self.option.is_active(config) && self.ignored_when.is_active(config)
    }

    fn counterpart(self, options: &[FormatOption]) -> Option<FormatOption> {
        if options.contains(&self.option) {
            Some(self.ignored_when)
        } else {
            options.contains(&self.ignored_when).then_some(self.option)
        }
    }

    pub fn describe(self, name: impl Fn(FormatOption) -> String) -> String {
        format!(
            "{} {}",
            name(self.option),
            cannot_be_combined_with(&name(self.ignored_when))
        )
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
        .filter_map(|rule| rule.counterpart(options))
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
    fn drop_ignored(self) -> Self {
        FORMAT_RULES.iter().fold(self, |mut config, rule| {
            if let (true, Some(id)) = (
                rule.ignored_when.is_active(&config),
                rule.option.format_setting(),
            ) {
                config.settings.reset(id);
            }
            config
        })
    }

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
