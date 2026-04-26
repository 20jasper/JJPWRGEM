use core::range::Range;
use std::{borrow::Cow, path::Path};

use crate::{
    Error, ErrorKind,
    tokens::{JsonCharOption, Token, TokenOption, TokenWithContext, lexical::JsonChar},
};
pub const EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE: &str = "the preceding key/value pair";
pub const INSERT_MISSING_CLOSED_BRACE_HELP: &str = "insert the missing closed brace";
pub const REMOVE_EXPONENT_HELP: &str = "remove the exponent";
pub const INSERT_EXPONENT_PLACEHOLDER_DIGIT_HELP: &str = "insert a placeholder digit";
pub const EXPONENT_PLACEHOLDER_DIGIT: &str = "5";

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
pub struct Diagnostic<'a> {
    pub message: String,
    pub range: Option<Range<usize>>,
    pub context: Vec<Context<'a>>,
    pub patches: Vec<Patch<'a>>,
    pub source: Source<'a>,
}

impl<'a> Diagnostic<'a> {
    pub fn new(
        message: String,
        context: Vec<Context<'a>>,
        patches: Vec<Patch<'a>>,
        source: Source<'a>,
        range: Option<Range<usize>>,
    ) -> Self {
        Self {
            message,
            context,
            patches,
            source,
            range,
        }
    }
}

fn error_source<'a>(error: &'a Error<'a>) -> Source<'a> {
    if error.source_name == "stdin" {
        Source::Stdin(error.source_text.as_str())
    } else {
        Source::File {
            source: error.source_text.as_str(),
            path: Path::new(error.source_name.as_str()),
        }
    }
}

fn exponent_patch_suggestions<'a>(
    exponent_range: Range<usize>,
    source: Source<'a>,
) -> Vec<Patch<'a>> {
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

impl<'a> From<&'a Error<'a>> for Vec<Patch<'a>> {
    fn from(error: &'a Error<'a>) -> Self {
        let source = error_source(error);
        match &error.kind {
            ErrorKind::ExpectedKey(
                TokenWithContext {
                    token: Token::Comma,
                    range,
                },
                TokenOption(Some(_)),
            ) => {
                vec![Patch::new(
                    "consider removing the trailing comma",
                    *range,
                    source,
                    "",
                )]
            }
            ErrorKind::ExpectedKey(
                TokenWithContext {
                    token: Token::Comma,
                    range,
                },
                TokenOption(None),
            ) => {
                vec![Patch::new(
                    "consider replacing the trailing comma with a closed curly brace",
                    *range,
                    source,
                    "}",
                )]
            }
            ErrorKind::ExpectedColon(ctx, found) => {
                let (message, replacement) = match found.0.as_ref() {
                    None => (
                        "insert colon, placeholder value, and closing curly brace",
                        r#": "garlic bread" }"#,
                    ),
                    Some(Token::Comma) | Some(Token::ClosedCurlyBrace) => {
                        ("insert colon and placeholder value", r#": "🐟🛹""#)
                    }
                    _ => ("insert the missing colon", ": "),
                };

                vec![Patch::new(
                    message,
                    ctx.range.end..ctx.range.end,
                    source,
                    replacement,
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
            ErrorKind::ExpectedCommaOrClosedCurlyBrace { range, found, .. } => {
                match found.0.as_ref() {
                    Some(Token::String(s)) => vec![Patch::new(
                        Cow::Owned(format!("is {s:?} a key? consider adding a comma")),
                        range.end..range.end,
                        source,
                        ",",
                    )],
                    None => vec![Patch::new(
                        INSERT_MISSING_CLOSED_BRACE_HELP,
                        range.end..range.end,
                        source,
                        "}",
                    )],
                    _ => Vec::new(),
                }
            }
            ErrorKind::ExpectedValue(ctx, tok_opt) => match (ctx, tok_opt.0.as_ref()) {
                (
                    Some(TokenWithContext {
                        token: Token::Comma,
                        range,
                    }),
                    Some(Token::ClosedSquareBracket),
                ) => vec![Patch::new(
                    "consider removing the trailing comma",
                    *range,
                    source,
                    "",
                )],
                (_, None) => vec![Patch::new(
                    "insert a placeholder value",
                    error.range.end..error.range.end,
                    source,
                    " \"rust is a must\"",
                )],
                (_, Some(Token::ClosedCurlyBrace)) => vec![Patch::new(
                    "consider adding the missing open curly brace",
                    error.range.end - 1..error.range.end,
                    source,
                    "{}",
                )],
                _ => Vec::new(),
            },
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
                number_range,
                maybe_c,
                ..
            } => {
                if maybe_c.0.is_none_or(|c| c.is_structural()) {
                    exponent_patch_suggestions(exponent_range.start..number_range.end, source)
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
            ErrorKind::ExpectedEntryOrClosedDelimiter {
                found: TokenOption(Some(_)),
                ..
            } => Vec::new(),
            ErrorKind::UnexpectedCharacter(_) => Vec::new(),
            ErrorKind::ExpectedHexDigit { .. } => Vec::new(),
            ErrorKind::InvalidEncoding(_) => Vec::new(),
            ErrorKind::ExpectedOpenBrace { .. } => Vec::new(),
            ErrorKind::ExpectedMinusOrDigit(_) => Vec::new(),
            ErrorKind::ExpectedKey(_, _) => Vec::new(),
        }
    }
}

impl<'a> From<&'a Error<'a>> for Vec<Context<'a>> {
    fn from(error: &'a Error<'a>) -> Self {
        let source = error_source(error);
        match &error.kind {
            ErrorKind::ExpectedKey(ctx, _)
            | ErrorKind::ExpectedColon(ctx, _)
            | ErrorKind::ExpectedEntryOrClosedDelimiter { open_ctx: ctx, .. }
            | ErrorKind::ExpectedValue(Some(ctx), _)
            | ErrorKind::ExpectedOpenBrace {
                context: Some(ctx), ..
            } => vec![Context::new(
                format!("expected due to {}", ctx.token),
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
                    format!("object opened here by {}", open_ctx.token),
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
            ErrorKind::ExpectedDigitAfterE {
                exponent_range,
                number_range,
                ..
            } => vec![Context::new(
                "exponent found here",
                exponent_range.start..number_range.end,
                source,
            )],
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

impl<'a> From<&'a Error<'a>> for Diagnostic<'a> {
    fn from(error: &'a Error<'a>) -> Self {
        Diagnostic {
            message: error.kind.to_string(),
            range: Some(error.range),
            context: error.into(),
            patches: error.into(),
            source: error_source(error),
        }
    }
}
