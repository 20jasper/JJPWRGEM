use core::ops::Range;

use displaydoc::Display;
use thiserror::Error;

use crate::tokens::{Token, TokenWithContext};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Display)]
pub enum ErrorKind {
    /// Unexpected character {0:?}
    UnexpectedCharacter(char),
    /// unexpected token {0:?} after json finished
    TokenAfterEnd(Token),
    /// expected key after comma, found {0:?}
    ExpectedKey(Option<Token>),
    /// expected colon after key, found {0:?}
    ExpectedColon(Option<Token>),
    /// expected json value, found {0:?}
    ExpectedValue(Option<Token>),
    /// expected key or closed curly brace, found {0:?}
    ExpectedKeyOrClosedCurlyBrace(Option<Token>),
    /// expected comma or closed curly brace, found {0:?}
    ExpectedCommaOrClosedCurlyBrace(Option<Token>),
    /// expected open curly curly brace, found {0:?}
    ExpectedOpenCurlyBrace(Option<Token>),
    /// expected quote, found {0:?}
    ExpectedQuote(Option<char>),
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
    kind: ErrorKind,
    // TODO temp for migration
    range: Range<usize>,
    /// 1 indexed line number
    line: usize,
    /// 1 indexed column number
    column: usize,
}

impl Error {
    pub fn new(kind: ErrorKind, range: Range<usize>, text: &str) -> Self {
        let (line, column) = get_line_and_column(text, range.clone());
        Self {
            kind,
            range,
            line,
            column,
        }
    }

    pub fn from_maybe_token_with_context(
        f: impl Fn(Option<Token>) -> ErrorKind,
        maybe_token: Option<TokenWithContext>,
        text: &str,
    ) -> Self {
        if let Some(TokenWithContext { token, range }) = maybe_token {
            Error::new(f(Some(token)), range, text)
        } else {
            Error::new(f(None), text.len()..text.len() + 1, text)
        }
    }
}

// TODO temp for migration
impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self {
            kind,
            range: 0..0,
            line: 0,
            column: 0,
        }
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
