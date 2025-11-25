use crate::{
    Error, ErrorKind,
    tokens::{JsonCharOption, Token, TokenOption, lexical::JsonChar},
};
use annotate_snippets::{Annotation, AnnotationKind, Group, Level, Snippet};
use core::ops::Range;
use std::{borrow::Cow, path::Path};
pub const EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE: &str = "the preceding key/value pair";
pub const INSERT_MISSING_CURLY_HELP: &str = "insert the missing curly brace";

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
    pub context: Vec<Context<'a>>,
    pub patches: Vec<Patch<'a>>,
    pub source: Source<'a>,
}

impl<'a> Diagnostic<'a> {
    #[allow(dead_code)]
    pub fn new(
        message: String,
        context: Vec<Context<'a>>,
        patches: Vec<Patch<'a>>,
        source: Source<'a>,
    ) -> Self {
        Self {
            message,
            context,
            patches,
            source,
        }
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

pub fn patches_from_error<'a>(error: &'a Error) -> Vec<Patch<'a>> {
    let source = error_source(error);
    let source_len = error.source_text.len();
    match &*error.kind {
        ErrorKind::ExpectedKey(ctx, TokenOption(Some(_))) => vec![Patch::new(
            "consider removing the trailing comma",
            ctx.range.clone(),
            source,
            "",
        )],
        ErrorKind::ExpectedKey(ctx, TokenOption(None)) => vec![Patch::new(
            "consider replacing the trailing comma with a closed curly brace",
            ctx.range.clone(),
            source,
            "}",
        )],
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
        ErrorKind::ExpectedKeyOrClosedCurlyBrace(_, TokenOption(None)) => vec![Patch::new(
            INSERT_MISSING_CURLY_HELP,
            error.range.end..error.range.end,
            source,
            "}",
        )],
        ErrorKind::ExpectedCommaOrClosedCurlyBrace { range, found, .. } => match found.0.as_ref() {
            Some(Token::String(s)) => vec![Patch::new(
                Cow::Owned(format!("is {s:?} a key? consider adding a comma")),
                range.end..range.end,
                source,
                ",",
            )],
            None => vec![Patch::new(
                INSERT_MISSING_CURLY_HELP,
                range.end..range.end,
                source,
                "}",
            )],
            _ => Vec::new(),
        },
        ErrorKind::ExpectedValue(_, tok_opt) => match tok_opt.0.as_ref() {
            None => vec![Patch::new(
                "insert a placeholder value",
                error.range.end..error.range.end,
                source,
                " \"rust is a must\"",
            )],
            Some(Token::ClosedCurlyBrace) => vec![Patch::new(
                "consider adding the missing open curly brace",
                error.range.end - 1..error.range.end,
                source,
                "{}",
            )],
            _ => Vec::new(),
        },
        ErrorKind::UnexpectedControlCharacterInString(escaped) => {
            vec![Patch::new(
                "replace the control character with its escaped form",
                error.range.clone(),
                source,
                escaped.to_string(),
            )]
        }
        ErrorKind::TokenAfterEnd(token) => {
            let start = error.range.start.min(source_len);
            let end = source_len;
            if start >= end {
                Vec::new()
            } else {
                vec![Patch::new(
                    format!("consider removing the trailing content (starting with {token})"),
                    start..end,
                    source,
                    "",
                )]
            }
        }
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
            vec![Patch::new(
                "remove the leading zeros",
                extra.clone(),
                source,
                "",
            )]
        }
        ErrorKind::ExpectedDigitAfterDot {
            maybe_c: JsonCharOption(None),
            number_ctx,
            ..
        } => {
            let patch_range = number_ctx.end..number_ctx.end;

            vec![Patch::new(
                "insert placeholder digit after the decimal point",
                patch_range,
                source,
                "0",
            )]
        }
        ErrorKind::ExpectedDigitAfterDot {
            maybe_c: JsonCharOption(Some(_)),
            ..
        }
        | ErrorKind::ExpectedKeyOrClosedCurlyBrace(_, TokenOption(Some(_)))
        | ErrorKind::UnexpectedCharacter(_)
        | ErrorKind::ExpectedOpenCurlyBrace(_, _)
        | ErrorKind::ExpectedQuote
        | ErrorKind::Custom(_) => Vec::new(),
        ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
            number_ctx: _,
            e_ctx,
            maybe_c,
        } => {
            let insertion = e_ctx.end..e_ctx.end;
            match maybe_c.0 {
                None => vec![Patch::new(
                    "add placeholder exponent digits",
                    insertion,
                    source,
                    "+1",
                )],
                _ => Vec::new(),
            }
        }
    }
}

pub fn context_from_error<'a>(error: &'a Error) -> Vec<Context<'a>> {
    let source = error_source(error);
    match &*error.kind {
        ErrorKind::ExpectedKey(ctx, _)
        | ErrorKind::ExpectedColon(ctx, _)
        | ErrorKind::ExpectedKeyOrClosedCurlyBrace(ctx, _)
        | ErrorKind::ExpectedValue(Some(ctx), _)
        | ErrorKind::ExpectedOpenCurlyBrace(Some(ctx), _) => vec![Context::new(
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
        ErrorKind::ExpectedPlusOrMinusOrDigitAfterE {
            number_ctx,
            e_ctx,
            maybe_c: _,
        } => vec![
            Context::new(
                "number with exponent found here",
                number_ctx.clone(),
                source,
            ),
            Context::new("exponent indicator found here", e_ctx.clone(), source),
        ],
        ErrorKind::ExpectedValue(None, _)
        | ErrorKind::UnexpectedCharacter(_)
        | ErrorKind::UnexpectedControlCharacterInString(_)
        | ErrorKind::TokenAfterEnd(_)
        | ErrorKind::ExpectedQuote
        | ErrorKind::Custom(_)
        | ErrorKind::ExpectedOpenCurlyBrace(None, _) => Vec::new(),
    }
}

pub fn diagnostic_from_error<'a>(error: &'a Error) -> Diagnostic<'a> {
    Diagnostic {
        message: error.kind.to_string(),
        context: context_from_error(error),
        patches: patches_from_error(error),
        source: error_source(error),
    }
}

impl Error {
    pub fn report<'a>(&'a self) -> Vec<Group<'a>> {
        let Diagnostic {
            message,
            context,
            patches,
            source,
        } = diagnostic_from_error(self);

        let annotations = std::iter::once(AnnotationKind::Primary.span(self.range.clone()))
            .chain(context.into_iter().map(Annotation::from));

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
