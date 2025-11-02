use displaydoc::Display;
use thiserror::Error;

use crate::tokens::Token;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Display, Error)]
pub enum Error {
    /// Json may not be empty
    Empty,
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
    /// {0}
    Custom(String),
}

impl<S> From<S> for Error
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self::Custom(value.into())
    }
}
