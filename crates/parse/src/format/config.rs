use std::fmt;

use super::{FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Parser {
    #[default]
    Json,
    Jsonlines,
}

impl Parser {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Jsonlines => "jsonlines",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    #[default]
    Pretty,
    Uglify,
}

impl FormatMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pretty => "pretty",
            Self::Uglify => "uglify",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InputSettings {
    pub parser: Parser,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FormatSettings {
    pub mode: FormatMode,
    pub preferred_width: Option<usize>,
    pub end_of_line: LineEnding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatSetting {
    Mode,
    PreferredWidth,
    EndOfLine,
}

impl FormatSetting {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Mode => "mode",
            Self::PreferredWidth => "preferred_width",
            Self::EndOfLine => "end_of_line",
        }
    }
}

impl FormatSettings {
    pub fn is_set(&self, id: FormatSetting) -> bool {
        match id {
            FormatSetting::Mode => self.mode != FormatMode::default(),
            FormatSetting::PreferredWidth => self.preferred_width.is_some(),
            FormatSetting::EndOfLine => self.end_of_line != LineEnding::default(),
        }
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

    pub fn setting_name(self) -> &'static str {
        match self {
            Self::Setting(id) => id.name(),
            Self::Mode(_) => FormatSetting::Mode.name(),
            Self::Parser(_) => "parser",
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
        let setting = self.setting_name().replace('_', " ");
        match self.value() {
            Some(value) => format!("{setting} {value}"),
            None => setting,
        }
    }

    pub fn for_setting(setting_name: &str) -> Vec<Self> {
        FORMAT_RULES
            .iter()
            .flat_map(|rule| [rule.option, rule.ignored_when])
            .filter(|option| option.setting_name() == setting_name)
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
        let mode = match config.settings.mode {
            FormatMode::Uglify => JsonMode::Uglify,
            FormatMode::Pretty => JsonMode::Prettify(
                PrettifyOptions::builder()
                    .maybe_preferred_width(config.settings.preferred_width)
                    .line_ending(config.settings.end_of_line)
                    .build(),
            ),
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
