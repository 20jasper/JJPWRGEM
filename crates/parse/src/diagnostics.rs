use core::range::Range;
use std::{borrow::Cow, path::Path};

use crate::{
    Error, ErrorKind,
    tokens::{ErrorToken, JsonCharOption, Token, TokenOption, TokenWithContext, lexical::JsonChar},
};
pub(crate) const EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE: &str = "the preceding key/value pair";
pub(crate) const INSERT_MISSING_CLOSED_BRACE_HELP: &str = "insert the missing closed brace";
pub(crate) const REMOVE_EXPONENT_HELP: &str = "remove the exponent";
pub(crate) const INSERT_EXPONENT_PLACEHOLDER_DIGIT_HELP: &str = "insert a placeholder digit";
pub(crate) const EXPONENT_PLACEHOLDER_DIGIT: &str = "5";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Context<'a> {
    pub message: Cow<'a, str>,
    pub span: Range<usize>,
    pub source: Source<'a>,
}

impl<'a> Context<'a> {
    fn new(message: impl Into<Cow<'a, str>>, span: Range<usize>, source: Source<'a>) -> Self {
        Self {
            message: message.into(),
            span,
            source,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Patch<'a> {
    pub message: Cow<'a, str>,
    pub span: Range<usize>,
    pub source: Source<'a>,
    pub replacement: Cow<'a, str>,
}

impl<'a> Patch<'a> {
    fn new(
        message: impl Into<Cow<'a, str>>,
        span: Range<usize>,
        source: Source<'a>,
        replacement: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            message: message.into(),
            span,
            source,
            replacement: replacement.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Source<'a> {
    Stdin(&'a str),
    File { source: &'a str, path: &'a Path },
}
#[derive(Debug)]
pub struct Diagnostic<'a> {
    pub message: String,
    pub range: Option<Range<usize>>,
    pub context: Vec<Context<'a>>,
    pub patches: Vec<Patch<'a>>,
    pub source: Source<'a>,
}

fn error_source(error: &Error) -> Source<'_> {
    if error.source_name == "stdin" {
        Source::Stdin(error.source_text.as_str())
    } else {
        Source::File {
            source: error.source_text.as_str(),
            path: Path::new(error.source_name.as_str()),
        }
    }
}

fn exponent_patch_suggestions(exponent_range: Range<usize>, source: Source<'_>) -> Vec<Patch<'_>> {
    vec![
        Patch::new(REMOVE_EXPONENT_HELP, exponent_range, source, ""),
        Patch::new(
            INSERT_EXPONENT_PLACEHOLDER_DIGIT_HELP,
            exponent_range.end..exponent_range.end,
            source,
            EXPONENT_PLACEHOLDER_DIGIT,
        ),
    ]
}

impl<'a> From<&'a Error> for Vec<Patch<'a>> {
    fn from(error: &'a Error) -> Self {
        let source = error_source(error);
        match &error.kind {
            ErrorKind::ExpectedKey(
                TokenWithContext {
                    token: Token::Comma,
                    range: comma_range,
                },
                TokenOption(Some(_)),
            )
            | ErrorKind::ExpectedValue(
                Some(TokenWithContext {
                    token: Token::Comma,
                    range: comma_range,
                }),
                TokenOption(Some(ErrorToken {
                    tag: Token::ClosedSquareBracket,
                    ..
                })),
            ) => vec![Patch::new(
                "consider removing the trailing comma",
                *comma_range,
                source,
                "",
            )],
            ErrorKind::ExpectedKey(
                TokenWithContext {
                    token: Token::Comma,
                    range: comma_range,
                },
                TokenOption(None),
            ) => vec![Patch::new(
                "consider replacing the trailing comma with a closed curly brace",
                *comma_range,
                source,
                "}",
            )],
            ErrorKind::ExpectedColon(ctx, TokenOption(None)) => {
                let r = ctx.range;
                vec![Patch::new(
                    "insert colon, placeholder value, and closing curly brace",
                    r.end..r.end,
                    source,
                    r#": "garlic bread" }"#,
                )]
            }
            ErrorKind::ExpectedColon(
                ctx,
                TokenOption(Some(ErrorToken {
                    tag: Token::Comma | Token::ClosedCurlyBrace,
                    ..
                })),
            ) => {
                let r = ctx.range;
                vec![Patch::new(
                    "insert colon and placeholder value",
                    r.end..r.end,
                    source,
                    r#": "🐟🛹""#,
                )]
            }
            ErrorKind::ExpectedColon(ctx, _) => {
                let r = ctx.range;
                vec![Patch::new(
                    "insert the missing colon",
                    r.end..r.end,
                    source,
                    ": ",
                )]
            }
            ErrorKind::ExpectedEntryOrClosedDelimiter {
                expected,
                found: TokenOption(None),
                ..
            } => vec![Patch::new(
                Cow::Owned(format!("insert the missing closed delimiter `{expected}`")),
                error.range.end..error.range.end,
                source,
                expected.to_string(),
            )],
            ErrorKind::ExpectedEntryOrClosedDelimiter {
                found: TokenOption(Some(_)),
                ..
            } => Vec::new(),
            ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                range,
                found:
                    TokenOption(Some(
                        t @ ErrorToken {
                            tag: Token::String, ..
                        },
                    )),
                ..
            } => {
                let s = t.content();
                vec![Patch::new(
                    Cow::Owned(format!("is {s} a key? consider adding a comma")),
                    range.end..range.end,
                    source,
                    ",",
                )]
            }
            ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                range,
                found: TokenOption(None),
                ..
            } => vec![Patch::new(
                INSERT_MISSING_CLOSED_BRACE_HELP,
                range.end..range.end,
                source,
                "}",
            )],
            ErrorKind::ExpectedCommaOrClosedCurlyBrace { .. } => Vec::new(),
            ErrorKind::ExpectedValue(_, TokenOption(None)) => vec![Patch::new(
                "insert a placeholder value",
                error.range.end..error.range.end,
                source,
                " \"rust is a must\"",
            )],
            ErrorKind::ExpectedValue(
                _,
                TokenOption(Some(ErrorToken {
                    tag: Token::ClosedCurlyBrace,
                    ..
                })),
            ) => vec![Patch::new(
                "consider adding the missing open curly brace",
                error.range.end - 1..error.range.end,
                source,
                "{}",
            )],
            ErrorKind::ExpectedValue(_, _) => Vec::new(),
            ErrorKind::UnexpectedControlCharacterInString(escaped) => vec![Patch::new(
                "replace the control character with its escaped form",
                error.range,
                source,
                escaped.to_string(),
            )],
            ErrorKind::TokenAfterEnd(token) => vec![Patch::new(
                format!("consider removing the trailing content (starting with {token})"),
                error.range.start..error.source_text.len(),
                source,
                "",
            )],
            ErrorKind::ExpectedDigitFollowingMinus(range, found) => {
                let patch_info = match found.0 {
                    None => ("insert placeholder digits after the minus sign", "194"),
                    Some(JsonChar('.')) => (
                        "did you mean to add a fraction? consider adding a 0 before the period",
                        "0",
                    ),
                    _ => return vec![],
                };
                let (message, replacement) = patch_info;
                {
                    vec![Patch::new(
                        message,
                        range.end..range.end,
                        source,
                        replacement,
                    )]
                }
            }
            ErrorKind::UnexpectedLeadingZero { extra, .. } => {
                vec![Patch::new("remove the leading zeros", *extra, source, "")]
            }
            ErrorKind::ExpectedDigitAfterDot {
                maybe_c: JsonCharOption(None),
                number_range,
                ..
            } => vec![Patch::new(
                "insert placeholder digit after the decimal point",
                number_range.end..number_range.end,
                source,
                "0",
            )],
            ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                e_range, maybe_c, ..
            } => {
                if maybe_c.0.is_none_or(|c| c.is_structural()) {
                    exponent_patch_suggestions(*e_range, source)
                } else {
                    Vec::new()
                }
            }
            ErrorKind::ExpectedDigitAfterE {
                exponent_range,
                maybe_c,
                ..
            } => {
                if maybe_c.0.is_none_or(|c| c.is_structural()) {
                    exponent_patch_suggestions(*exponent_range, source)
                } else {
                    Vec::new()
                }
            }
            ErrorKind::ExpectedQuote { string_range, .. } => vec![Patch::new(
                "insert the missing closing quote",
                string_range.end..string_range.end,
                source,
                "\"",
            )],
            ErrorKind::ExpectedEscape {
                maybe_c,
                slash_range,
                ..
            } => match maybe_c.0.as_ref() {
                Some(c) if c.is_control() => {
                    vec![Patch::new(
                        "escape the control character",
                        slash_range.start..error.range.end,
                        source,
                        c.escape(),
                    )]
                }
                _ => {
                    vec![Patch::new(
                        "remove unnecessary escape slash",
                        *slash_range,
                        source,
                        "",
                    )]
                }
            },

            ErrorKind::ExpectedDigitAfterDot { .. } => Vec::new(),
            ErrorKind::UnexpectedCharacter(_) => Vec::new(),
            ErrorKind::ExpectedHexDigit { .. } => Vec::new(),
            ErrorKind::InvalidEncoding(_) => Vec::new(),
            ErrorKind::ExpectedOpenBrace { .. } => Vec::new(),
            ErrorKind::ExpectedMinusOrDigit(_) => Vec::new(),
            ErrorKind::ExpectedKey(_, _) => Vec::new(),
        }
    }
}

impl<'a> From<&'a Error> for Vec<Context<'a>> {
    fn from(error: &'a Error) -> Self {
        let source = error_source(error);
        match &error.kind {
            ErrorKind::ExpectedKey(ctx, _)
            | ErrorKind::ExpectedColon(ctx, _)
            | ErrorKind::ExpectedEntryOrClosedDelimiter { open_ctx: ctx, .. }
            | ErrorKind::ExpectedValue(Some(ctx), _)
            | ErrorKind::ExpectedOpenBrace {
                context: Some(ctx), ..
            } => vec![Context::new(
                format!(
                    "expected due to `{}`",
                    &error.source_text[ctx.range.start..ctx.range.end]
                ),
                ctx.range,
                source,
            )],
            ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                range, open_ctx, ..
            } => vec![
                Context::new(
                    format!("expected due to {EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE}"),
                    *range,
                    source,
                ),
                Context::new(
                    format!(
                        "object opened here by `{}`",
                        &error.source_text[open_ctx.range.start..open_ctx.range.end]
                    ),
                    open_ctx.range,
                    source,
                ),
            ],
            ErrorKind::ExpectedDigitFollowingMinus(range, _) => {
                vec![Context::new("minus sign found here", *range, source)]
            }
            ErrorKind::UnexpectedLeadingZero { initial, .. } => {
                vec![Context::new("first zero found here", *initial, source)]
            }
            ErrorKind::ExpectedDigitAfterDot {
                dot_range,
                number_range,
                ..
            } => vec![
                Context::new("decimal point found here", *dot_range, source),
                Context::new("number found here", *number_range, source),
            ],
            ErrorKind::ExpectedDigitAfterE { exponent_range, .. } => {
                vec![Context::new("exponent found here", *exponent_range, source)]
            }
            ErrorKind::ExpectedPlusOrMinusOrDigitAfterE { e_range, .. } => {
                vec![Context::new("exponent found here", *e_range, source)]
            }
            ErrorKind::ExpectedQuote { open_range, .. } => vec![Context::new(
                "opening quote found here",
                *open_range,
                source,
            )],
            ErrorKind::ExpectedEscape {
                slash_range,
                quote_range,
                ..
            } => vec![
                Context::new("escape slash found here", *slash_range, source),
                Context::new("opening quote found here", *quote_range, source),
            ],

            ErrorKind::ExpectedHexDigit {
                quote_range,
                slash_range,
                u_range,
                ..
            } => vec![
                Context::new("opening quote found here", *quote_range, source),
                Context::new(
                    "\\u escape started here",
                    slash_range.start..u_range.end,
                    source,
                ),
            ],
            ErrorKind::ExpectedValue(None, _) => Vec::new(),
            ErrorKind::UnexpectedCharacter(_) => Vec::new(),
            ErrorKind::UnexpectedControlCharacterInString(_) => Vec::new(),
            ErrorKind::TokenAfterEnd(_) => Vec::new(),
            ErrorKind::InvalidEncoding(_) => Vec::new(),
            ErrorKind::ExpectedMinusOrDigit(_) => Vec::new(),
            ErrorKind::ExpectedOpenBrace { context: None, .. } => Vec::new(),
        }
    }
}

impl<'a> From<&'a Error> for Diagnostic<'a> {
    fn from(error: &'a Error) -> Self {
        Diagnostic {
            message: error.message(),
            range: Some(error.range),
            context: error.into(),
            patches: error.into(),
            source: error_source(error),
        }
    }
}
