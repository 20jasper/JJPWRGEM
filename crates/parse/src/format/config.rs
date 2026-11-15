use std::fmt;

use super::{
    DEFAULT_PREFERRED_WIDTH, FormatRequest, JsonMode, JsonlinesOptions, LineEnding, PrettifyOptions,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Parser {
    #[default]
    Json,
    Jsonlines,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatConfig {
    pub parser: Parser,
    pub uglify: bool,
    pub preferred_width: usize,
    pub line_ending: LineEnding,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            parser: Parser::default(),
            uglify: false,
            preferred_width: DEFAULT_PREFERRED_WIDTH,
            line_ending: LineEnding::default(),
        }
    }
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
    fn is_active(self, config: &FormatConfig) -> bool {
        match self {
            Self::Uglify => config.uglify,
            Self::Jsonlines => config.parser == Parser::Jsonlines,
            Self::PreferredWidth => config.preferred_width != DEFAULT_PREFERRED_WIDTH,
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
                    .preferred_width(self.preferred_width)
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
        DEFAULT_PREFERRED_WIDTH,
        Ok(FormatRequest::Json(prettify(DEFAULT_PREFERRED_WIDTH)))
    )]
    #[case(Parser::Json, false, WIDTH, Ok(FormatRequest::Json(prettify(WIDTH))))]
    #[case(
        Parser::Json,
        true,
        DEFAULT_PREFERRED_WIDTH,
        Ok(FormatRequest::Json(JsonMode::Uglify))
    )]
    #[case(
        Parser::Jsonlines,
        false,
        DEFAULT_PREFERRED_WIDTH,
        Ok(FormatRequest::Jsonlines(
            JsonlinesOptions::builder()
                .line_ending(LineEnding::CrLf)
                .build()
        ))
    )]
    #[case(
        Parser::Jsonlines,
        true,
        DEFAULT_PREFERRED_WIDTH,
        Err(ConfigError(Rule::Conflicts(FormatOption::Uglify, FormatOption::Jsonlines)))
    )]
    #[case(
        Parser::Jsonlines,
        false,
        WIDTH,
        Err(ConfigError(Rule::Conflicts(FormatOption::PreferredWidth, FormatOption::Jsonlines)))
    )]
    #[case(
        Parser::Json,
        true,
        WIDTH,
        Err(ConfigError(Rule::Conflicts(FormatOption::Uglify, FormatOption::PreferredWidth)))
    )]
    fn validate(
        #[case] parser: Parser,
        #[case] uglify: bool,
        #[case] preferred_width: usize,
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
