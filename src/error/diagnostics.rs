use crate::{
    Error, ErrorKind,
    tokens::{JsonCharOption, Token, TokenOption, TokenWithContext, lexical::JsonChar},
};
use annotate_snippets::{Annotation, AnnotationKind, Group, Level, Snippet};
use core::ops::Range;
use std::{borrow::Cow, path::Path};
pub const EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE: &str = "the preceding key/value pair";
pub const INSERT_MISSING_CLOSED_BRACE_HELP: &str = "insert the missing closed brace";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Context<'a> {
    message: Cow<'a, str>,
    span: Range<usize>,
    source: Source<'a>,
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
    message: Cow<'a, str>,
    span: Range<usize>,
    source: Source<'a>,
    replacement: Cow<'a, str>,
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

impl<'a> From<Patch<'a>> for annotate_snippets::Patch<'a> {
    fn from(patch: Patch<'a>) -> Self {
        annotate_snippets::Patch::new(patch.span, patch.replacement)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Source<'a> {
    Stdin(&'a str),
    File { source: &'a str, path: &'a Path },
}
impl<'a, T: Clone> From<Source<'a>> for Snippet<'a, T> {
    fn from(val: Source<'a>) -> Self {
        let (source, path) = match val {
            Source::Stdin(src) => (src, "stdin"),
            Source::File { source, path } => (
                source,
                path.to_str()
                    .expect("diagnostic paths should be valid utf8"),
            ),
        };
        Snippet::source(source).path(path)
    }
}

impl<'a> From<Context<'a>> for Annotation<'a> {
    fn from(ctx: Context<'a>) -> Self {
        let Context {
            message,
            span,
            source: _,
        } = ctx;
        AnnotationKind::Context.span(span).label(message)
    }
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

    fn report(self) -> Vec<Group<'a>> {
        let Diagnostic {
            message,
            context,
            patches,
            source,
            range,
        } = self;

        let annotations = if let Some(range) = range {
            std::iter::once(AnnotationKind::Primary.span(range))
                .chain(context.into_iter().map(Annotation::from))
                .collect()
        } else {
            vec![]
        };

        let error_group = Level::ERROR
            .primary_title(message)
            .element(Snippet::from(source).annotations(annotations));
        let patch_group = patches.into_iter().map(|patch| {
            Level::HELP
                .primary_title(patch.message.clone())
                .element(Snippet::from(source).patches(vec![patch.into()]))
        });

        std::iter::once(error_group).chain(patch_group).collect()
    }
}

fn error_source<'a>(error: &'a Error) -> Source<'a> {
    if error.source_name == "stdin" {
        Source::Stdin(error.source_text.as_str())
    } else {
        Source::File {
            source: error.source_text.as_str(),
            path: Path::new(error.source_name.as_str()),
        }
    }
}

impl<'a> From<&'a Error> for Vec<Patch<'a>> {
    fn from(error: &'a Error) -> Self {
        let source = error_source(error);
        match &*error.kind {
                ErrorKind::ExpectedKey(
                    TokenWithContext {
                        token: Token::Comma,
                        range,
                    },
                    TokenOption(Some(_)),
                ) => {
                    vec![Patch::new(
                        "consider removing the trailing comma",
                        range.clone(),
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
                        range.clone(),
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
                ErrorKind::ExpectedCommaOrClosedCurlyBrace { range, found, .. } => match found.0.as_ref() {
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
                },
                ErrorKind::ExpectedValue(ctx, tok_opt) => match (ctx, tok_opt.0.as_ref()) {
                    (
                        Some(TokenWithContext {
                            token: Token::Comma,
                            range,
                        }),
                        Some(Token::ClosedSquareBracket),
                    ) => vec![Patch::new(
                        "consider removing the trailing comma",
                        range.clone(),
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
                    error.range.clone(),
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
                ErrorKind::UnexpectedLeadingZero { extra, .. } => vec![Patch::new(
                    "remove the leading zeros",
                    extra.clone(),
                    source,
                    "",
                )],
                ErrorKind::ExpectedDigitAfterDot {
                    maybe_c: JsonCharOption(None),
                    number_ctx,
                    ..
                } => vec![Patch::new(
                    "insert placeholder digit after the decimal point",
                    number_ctx.end..number_ctx.end,
                    source,
                    "0",
                )],
                ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                    e_ctx,
                    maybe_c: JsonCharOption(None),
                    ..
                } => vec![Patch::new(
                    "add placeholder exponent digits",
                    e_ctx.end..e_ctx.end,
                    source,
                    "+1",
                )],
                ErrorKind::ExpectedDigitAfterE {
                    maybe_c: JsonCharOption(None),
                    number_ctx,
                    ..
                } => vec![Patch::new(
                    "add a digit after the exponent sign",
                    number_ctx.end..number_ctx.end,
                    source,
                    "0",
                )],
                | ErrorKind::ExpectedKey(_, _)
                        // reachable?
                | ErrorKind::InvalidEncoding
                | ErrorKind::ExpectedDigitAfterE { .. }
                | ErrorKind::ExpectedDigitAfterDot { .. }
                | ErrorKind::ExpectedPlusOrMinusOrDigitAfterE { .. }
                | ErrorKind::ExpectedEntryOrClosedDelimiter {
                    found: TokenOption(Some(_)),
                    ..
                }
                | ErrorKind::UnexpectedCharacter(_)
                | ErrorKind::ExpectedOpenBrace { .. }
                | ErrorKind::ExpectedQuote
                | ErrorKind::ExpectedMinusOrDigit(_) => Vec::new(),
            }
    }
}

impl<'a> From<&'a Error> for Vec<Context<'a>> {
    fn from(error: &'a Error) -> Self {
        let source = error_source(error);
        match &*error.kind {
            ErrorKind::ExpectedKey(ctx, _)
            | ErrorKind::ExpectedColon(ctx, _)
            | ErrorKind::ExpectedEntryOrClosedDelimiter { open_ctx: ctx, .. }
            | ErrorKind::ExpectedValue(Some(ctx), _)
            | ErrorKind::ExpectedOpenBrace {
                context: Some(ctx), ..
            } => vec![Context::new(
                format!("expected due to {}", ctx.token),
                ctx.range.clone(),
                source,
            )],
            ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                range, open_ctx, ..
            } => vec![
                Context::new(
                    format!("expected due to {EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE}"),
                    range.clone(),
                    source,
                ),
                Context::new(
                    format!("object opened here by {}", open_ctx.token),
                    open_ctx.range.clone(),
                    source,
                ),
            ],
            ErrorKind::ExpectedDigitFollowingMinus(range, _) => {
                vec![Context::new("minus sign found here", range.clone(), source)]
            }
            ErrorKind::UnexpectedLeadingZero { initial, .. } => {
                vec![Context::new(
                    "first zero found here",
                    initial.clone(),
                    source,
                )]
            }
            ErrorKind::ExpectedDigitAfterDot {
                dot_ctx,
                number_ctx,
                ..
            } => vec![
                Context::new("decimal point found here", dot_ctx.clone(), source),
                Context::new("number found here", number_ctx.clone(), source),
            ],
            ErrorKind::ExpectedDigitAfterE {
                number_ctx,
                exponent_ctx,
                maybe_c: _,
            }
            | ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
                number_ctx,
                e_ctx: exponent_ctx,
                maybe_c: _,
            } => vec![
                Context::new(
                    "number with exponent found here",
                    number_ctx.clone(),
                    source,
                ),
                Context::new(
                    "exponent indicator found here",
                    exponent_ctx.clone(),
                    source,
                ),
            ],
            ErrorKind::InvalidEncoding
            | ErrorKind::ExpectedValue(None, _)
            | ErrorKind::UnexpectedCharacter(_)
            | ErrorKind::UnexpectedControlCharacterInString(_)
            | ErrorKind::TokenAfterEnd(_)
            | ErrorKind::ExpectedQuote
            | ErrorKind::ExpectedMinusOrDigit(_)
            | ErrorKind::ExpectedOpenBrace { context: None, .. } => Vec::new(),
        }
    }
}

impl<'a> From<&'a Error> for Diagnostic<'a> {
    fn from(error: &'a Error) -> Self {
        Diagnostic {
            message: error.kind.to_string(),
            range: Some(error.range.clone()),
            context: error.into(),
            patches: error.into(),
            source: error_source(error),
        }
    }
}

impl Error {
    pub fn report<'a>(&'a self) -> Vec<Group<'a>> {
        Diagnostic::from(self).report()
    }

    pub fn report_invalid_encoding<'a>(source: Source<'a>) -> Vec<Group<'a>> {
        Diagnostic {
            message: ErrorKind::InvalidEncoding.to_string(),
            source,
            range: None,
            patches: vec![],
            context: vec![],
        }
        .report()
    }
}
