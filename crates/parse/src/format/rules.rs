use std::fmt;

use super::config::{FormatConfig, FormatMode, FormatSetting, InputSetting, Parser};
use crate::settings::Setting;

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
        match self.value() {
            Some(value) => format!("{setting} {value}"),
            None => setting,
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

    pub fn ignored_when(row: &Setting) -> impl Iterator<Item = Self> {
        FORMAT_RULES
            .iter()
            .filter(move |rule| rule.option.setting().name == row.name)
            .map(|rule| rule.ignored_when)
    }
}

fn cannot_be_combined_with(others: &str) -> String {
    format!("cannot be combined with {others}")
}

impl Rule {
    pub(super) fn is_broken(self, config: &FormatConfig) -> bool {
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
