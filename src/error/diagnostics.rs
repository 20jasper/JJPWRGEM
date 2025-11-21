use crate::{
    Error, ErrorKind,
    tokens::{Token, TokenOption},
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
        ErrorKind::ExpectedKey(ctx, _) => vec![Patch::new(
            "consider removing the trailing comma",
            ctx.range.clone(),
            source,
            "",
        )],
        ErrorKind::ExpectedColon(ctx, found) => {
            let (message, replacement) = match found.0.as_ref() {
                None => (
                    "insert colon, placeholder value, and closing curly brace",
                    r#": "garlic bread" }"#,
                ),
                Some(Token::Comma) | Some(Token::ClosedCurlyBrace) => {
                    ("insert colon and placeholder key", r#": "🐟🛹""#)
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
                escaped,
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
        ErrorKind::ExpectedKeyOrClosedCurlyBrace(_, TokenOption(Some(_)))
        | ErrorKind::UnexpectedCharacter(_)
        | ErrorKind::ExpectedOpenCurlyBrace(_, _)
        | ErrorKind::ExpectedQuote
        | ErrorKind::Custom(_) => Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parse_str;
    use crate::test_json;
    use crate::tokens::Token;
    use core::ops::Range;
    use rstest::rstest;

    type ContextExpectation = (Range<usize>, String);
    type ContextExpectations = Vec<ContextExpectation>;

    fn context_case<M, I>(json: &'static str, contexts: I) -> (&'static str, ContextExpectations)
    where
        I: IntoIterator<Item = (Range<usize>, M)>,
        M: ToString,
    {
        (
            json,
            contexts
                .into_iter()
                .map(|(range, message)| (range, message.to_string()))
                .collect(),
        )
    }

    #[rstest]
    #[case(context_case(
        test_json::OBJECT_MISSING_COLON_WITH_COMMA,
        vec![(1..5, Token::String("hi".into()))],
    ))]
    #[case(context_case(
        test_json::OBJECT_MISSING_COLON_WITH_LEADING_WHITESPACE,
        vec![(3..7, Token::String("hi".into()))],
    ))]
    #[case(context_case(
        test_json::OBJECT_MISSING_COLON,
        vec![(1..5, Token::String("hi".into()))],
    ))]
    #[case(context_case(
        test_json::OBJECT_MISSING_VALUE,
        vec![(5..6, Token::Colon)],
    ))]
    #[case(context_case::<&str, Vec<(Range<usize>, &str)>>(test_json::CLOSED_CURLY, vec![]))]
    #[case(context_case::<&str, Vec<(Range<usize>, &str)>>(test_json::DOUBLE_QUOTE, vec![]))]
    #[case(context_case::<&str, Vec<(Range<usize>, &str)>>(
        test_json::OBJECT_WITH_LINE_BREAK_VALUE,
        vec![],
    ))]
    #[case(context_case(
        test_json::OBJECT_DOUBLE_OPEN_CURLY,
        vec![(0..1, Token::OpenCurlyBrace)],
    ))]
    #[case(context_case(
        test_json::OBJECT_OPEN_CURLY,
        vec![(0..1, Token::OpenCurlyBrace)],
    ))]
    #[case(context_case(
        test_json::OBJECT_MISSING_COMMA_BETWEEN_VALUES,
        vec![
            (
                5..11,
                EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE,
            ),
            (0..1, "object opened here"),
        ],
    ))]
    #[case(context_case(
        test_json::OBJECT_MISSING_COMMA_OR_CLOSING_WITH_WHITESPACE,
        vec![
            (
                5..11,
                EXPECTED_COMMA_OR_CLOSED_CURLY_MESSAGE,
            ),
            (0..1, "object opened here"),
        ],
    ))]
    #[case(context_case(
        test_json::OBJECT_TRAILING_COMMA_WITH_CLOSED,
        vec![(11..12, Token::Comma)],
    ))]
    #[case(context_case(
        test_json::OBJECT_TRAILING_COMMA,
        vec![(11..12, Token::Comma)],
    ))]
    #[case(context_case::<&str, Vec<(Range<usize>, &str)>>(test_json::OBJECT_EMPTY_THEN_OPEN, vec![]))]
    fn diagnostic_contexts_match_reported(
        #[case] (json, expected_ctx): (&'static str, ContextExpectations),
    ) {
        let error = parse_str(json).expect_err("expected parse error");
        let contexts = context_from_error(&error);

        for ((expected_span, expected_message_fragment), context) in
            expected_ctx.iter().zip(contexts.iter())
        {
            assert_eq!(&context.span, expected_span,);
            let message = context.message.as_ref();
            assert!(
                message.contains(expected_message_fragment.as_str()),
                "message `{message}` did not contain `{expected_message_fragment}`",
            );
        }

        assert_eq!(
            contexts.len(),
            expected_ctx.len(),
            "wrong amount of contexts"
        );
    }
}
