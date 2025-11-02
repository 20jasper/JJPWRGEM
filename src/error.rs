use displaydoc::Display;
use thiserror::Error;

use crate::tokens::Token;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Display, Error)]
pub enum Error {
    /// Json may not be empty
    Empty,
    /// unmatched character {0:?}
    Unmatched(Token),
    /// Unexpected token {0:?}
    UnexpectedToken(Token),
    /// Unexpected character {0:?}
    UnexpectedCharacter(char),
    /// unexpected token {0:?} after json finished
    TokenAfterEnd(Token),
    /// expected key after comma
    ExpectedKey,
    /// expected colon after key, found {0:?}
    ExpectedColon(Token),
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
