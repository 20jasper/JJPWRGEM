use displaydoc::Display;

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

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    /// uglify cannot be combined with jsonlines
    UglifyWithJsonlines,
    /// a preferred width cannot be combined with jsonlines
    PreferredWidthWithJsonlines,
    /// uglify cannot be combined with a preferred width
    PreferredWidthWithUglify,
}

impl std::error::Error for ConfigError {}

impl FormatConfig {
    pub fn validate(self) -> Result<FormatRequest, ConfigError> {
        if self.parser == Parser::Jsonlines {
            if self.uglify {
                return Err(ConfigError::UglifyWithJsonlines);
            }
            if self.preferred_width != DEFAULT_PREFERRED_WIDTH {
                return Err(ConfigError::PreferredWidthWithJsonlines);
            }
        }
        if self.uglify && self.preferred_width != DEFAULT_PREFERRED_WIDTH {
            return Err(ConfigError::PreferredWidthWithUglify);
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
        Err(ConfigError::UglifyWithJsonlines)
    )]
    #[case(
        Parser::Jsonlines,
        false,
        WIDTH,
        Err(ConfigError::PreferredWidthWithJsonlines)
    )]
    #[case(Parser::Json, true, WIDTH, Err(ConfigError::PreferredWidthWithUglify))]
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
