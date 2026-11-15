use std::fmt;

use super::{FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Parser {
    #[default]
    Json,
    Jsonlines,
}

const PREFERRED_WIDTH_SETTING: &str = "preferredWidth";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase", default))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FormatConfig {
    #[cfg_attr(feature = "serde", serde(skip))]
    pub parser: Parser,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub uglify: bool,
    /// Preferred maximum line width. Note this is not a hard maximum width
    #[cfg_attr(
        feature = "schema",
        schemars(extend("type" = "integer", "default" = super::DEFAULT_PREFERRED_WIDTH))
    )]
    pub preferred_width: Option<usize>,
    #[cfg_attr(feature = "serde", serde(rename = "endOfLine"))]
    /// Line ending to use when formatting output
    pub line_ending: LineEnding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKind {
    Integer {
        default: usize,
    },
    Choice {
        values: &'static [&'static str],
        default: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Setting {
    pub name: &'static str,
    pub flag: &'static str,
    pub description: &'static str,
    pub kind: SettingKind,
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
    pub const ALL: [Self; 3] = [Self::Uglify, Self::Jsonlines, Self::PreferredWidth];

    pub fn setting(self) -> Option<&'static str> {
        match self {
            Self::PreferredWidth => Some(PREFERRED_WIDTH_SETTING),
            Self::Uglify | Self::Jsonlines => None,
        }
    }

    fn is_active(self, config: &FormatConfig) -> bool {
        match self {
            Self::Uglify => config.uglify,
            Self::Jsonlines => config.parser == Parser::Jsonlines,
            Self::PreferredWidth => config.preferred_width.is_some(),
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
                    .maybe_preferred_width(self.preferred_width)
                    .line_ending(self.line_ending)
                    .build(),
            )
        };
        Ok(match self.parser {
            Parser::Json => FormatRequest::Json(mode),
            Parser::Jsonlines => FormatRequest::Jsonlines(
                JsonlinesOptions::builder()
                    .line_ending(self.line_ending)
                    .build(),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::format::DEFAULT_PREFERRED_WIDTH;

    const WIDTH: usize = 40;

    fn prettify(width: usize) -> JsonMode {
        JsonMode::Prettify(
            PrettifyOptions::builder()
                .preferred_width(width)
                .line_ending(LineEnding::CrLf)
                .build(),
        )
    }

    #[rstest]
    #[case(
        Parser::Json,
        false,
        None,
        Ok(FormatRequest::Json(prettify(DEFAULT_PREFERRED_WIDTH)))
    )]
    #[case(
        Parser::Json,
        false,
        Some(WIDTH),
        Ok(FormatRequest::Json(prettify(WIDTH)))
    )]
    #[case(Parser::Json, true, None, Ok(FormatRequest::Json(JsonMode::Uglify)))]
    #[case(
        Parser::Jsonlines,
        false,
        None,
        Ok(FormatRequest::Jsonlines(
            JsonlinesOptions::builder()
                .line_ending(LineEnding::CrLf)
                .build()
        ))
    )]
    #[case(
        Parser::Jsonlines,
        true,
        None,
        Err(ConfigError(Rule::Conflicts(FormatOption::Uglify, FormatOption::Jsonlines)))
    )]
    #[case(
        Parser::Jsonlines,
        false,
        Some(WIDTH),
        Err(ConfigError(Rule::Conflicts(FormatOption::PreferredWidth, FormatOption::Jsonlines)))
    )]
    #[case(
        Parser::Json,
        true,
        Some(WIDTH),
        Err(ConfigError(Rule::Conflicts(FormatOption::Uglify, FormatOption::PreferredWidth)))
    )]
    fn validate(
        #[case] parser: Parser,
        #[case] uglify: bool,
        #[case] preferred_width: Option<usize>,
        #[case] expected: Result<FormatRequest, ConfigError>,
    ) {
        let config = FormatConfig {
            parser,
            uglify,
            preferred_width,
            line_ending: LineEnding::CrLf,
        };
        assert_eq!(config.validate(), expected);
    }
}
