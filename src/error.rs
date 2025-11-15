use annotate_snippets::{Annotation, AnnotationKind, Group, Level, Patch, Snippet};
use core::ops::Range;
use displaydoc::Display;
use thiserror::Error;

use crate::tokens::{Token, TokenOption, TokenWithContext, trim_end_whitespace};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Display, Clone)]
pub enum ErrorKind {
    /// unexpected character `{0:?}`. expected start of a json value
    UnexpectedCharacter(char),
    /// unexpected unescaped control character `{0:?}` in string literal
    UnexpectedControlCharacterInString(char),
    /// unexpected token {0} after json finished
    TokenAfterEnd(Token),
    /// expected key, found {1}
    ExpectedKey(TokenWithContext, TokenOption),
    /// expected colon after key, found {1}
    ExpectedColon(TokenWithContext, TokenOption),
    /// expected json value, found {1}
    ExpectedValue(Option<TokenWithContext>, TokenOption),
    /// expected key or closed curly brace, found {1}
    ExpectedKeyOrClosedCurlyBrace(TokenWithContext, TokenOption),
    /// expected comma or closed curly brace, found {0}
    ExpectedCommaOrClosedCurlyBrace(TokenOption),
    /// expected open curly curly brace, found {0}
    ExpectedOpenCurlyBrace(TokenOption),
    /// expected quote before end of input
    ExpectedQuote,
    /// {0}
    Custom(String),
}

impl<S> From<S> for ErrorKind
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self::Custom(value.into())
    }
}

#[derive(Debug, PartialEq, Eq, Display, Error)]
/// {kind} at line {line} column {column}
pub struct Error {
    kind: Box<ErrorKind>,
    range: Range<usize>,
    /// 1 indexed line number
    line: usize,
    /// 1 indexed column number
    column: usize,
    source_text: String,
    source_name: String,
}

impl Error {
    pub fn new(kind: ErrorKind, range: Range<usize>, text: &str) -> Self {
        // TODO take this as a param or have some sort of context
        let source_name = "stdin".into();
        let (line, column) = get_line_and_column(text, range.clone());
        Self {
            kind: kind.into(),
            range,
            line,
            column,
            source_text: text.into(),
            source_name,
        }
    }

    pub fn from_unterminated(kind: ErrorKind, text: &str) -> Self {
        let trimmed = trim_end_whitespace(text);
        // TODO handle multibyte characters properly
        // text.char_indices().rev()
        Self::new(kind, trimmed.len().saturating_sub(1)..trimmed.len(), text)
    }

    pub fn from_maybe_token_with_context(
        f: impl Fn(TokenOption) -> ErrorKind,
        maybe_token: Option<TokenWithContext>,
        text: &str,
    ) -> Self {
        if let Some(TokenWithContext { token, range }) = maybe_token {
            Error::new(f(Some(token).into()), range, text)
        } else {
            Error::from_unterminated(f(None.into()), text)
        }
    }

    fn snippet<T>(&'_ self) -> Snippet<'_, T>
    where
        T: Clone,
    {
        Snippet::source(&self.source_text).path(&self.source_name)
    }

    fn report_patches(&'_ self) -> Vec<Group<'_>> {
        let (title, patches) = match *self.kind {
            ErrorKind::ExpectedKeyOrClosedCurlyBrace(_, _) => {
                let patch = Patch::new(self.range.end..self.range.end, "}");
                ("consider closing the unclosed curly brace", vec![patch])
            }
            _ => return vec![],
        };

        vec![
            Level::HELP
                .primary_title(title)
                .element(self.snippet().patches(patches)),
        ]
    }

    fn report_ctx(&'_ self) -> Option<Annotation<'_>> {
        let ctx = match &*self.kind {
            ErrorKind::ExpectedKey(ctx, _)
            | ErrorKind::ExpectedColon(ctx, _)
            | ErrorKind::ExpectedKeyOrClosedCurlyBrace(ctx, _)
            | ErrorKind::ExpectedValue(Some(ctx), _) => ctx,
            _ => return None,
        };

        let (range, label) = (ctx.range.clone(), format!("expected due to {}", ctx.token));
        Some(AnnotationKind::Context.span(range).label(label))
    }

    fn report_error(&'_ self) -> Group<'_> {
        let annotations = [
            Some(AnnotationKind::Primary.span(self.range.clone())),
            self.report_ctx(),
        ]
        .into_iter()
        .flatten();

        Level::ERROR
            .primary_title(self.kind.to_string())
            .element(self.snippet().annotations(annotations))
    }

    pub fn report<'a>(&'a self) -> Vec<Group<'a>> {
        std::iter::once(self.report_error())
            .chain(self.report_patches())
            .collect()
    }
}

fn get_line_and_column(text: &str, range: Range<usize>) -> (usize, usize) {
    let to_search = if let Some(to_search) = text.get(..=range.start) {
        to_search
    } else {
        return (1, 1);
    };

    let lines = to_search.lines().count();
    let column = to_search
        .lines()
        .last()
        .expect("to_search will never be empty")
        .chars()
        .count();
    (lines, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case("", 0..0, (1,1))]
    #[case("1\n2\n3", 0..1, (1,1))]
    #[case("1\n2\n3", 2..3, (2,1))]
    #[case("1\n234", 3..4, (2,2))]
    fn gets_line_and_column(
        #[case] text: &str,
        #[case] range: Range<usize>,
        #[case] expected: (usize, usize),
    ) {
        assert_eq!(get_line_and_column(text, range), expected);
    }
}
