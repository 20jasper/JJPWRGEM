pub mod diagnostics;

use crate::tokens::CharWithContext;
use crate::tokens::lexical::trim_end_whitespace;
use crate::tokens::{JsonCharOption, Token, TokenOption, TokenWithContext, lexical::JsonChar};
use core::fmt::Display;
use core::ops::{Deref, Range};
use displaydoc::Display;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Display, Clone)]
pub enum ErrorKind {
    // array/object
    /// expected key, found {1}
    ExpectedKey(TokenWithContext, TokenOption),
    /// expected colon after key, found {1}
    ExpectedColon(TokenWithContext, TokenOption),
    /// expected json value, found {1}
    ExpectedValue(Option<TokenWithContext>, TokenOption),
    /// expected entry or closed delimiter `{expected}`, found {found}
    ExpectedEntryOrClosedDelimiter {
        open_ctx: TokenWithContext,
        expected: JsonChar,
        found: TokenOption,
    },
    /// expected comma or closed curly brace, found {found}
    ExpectedCommaOrClosedCurlyBrace {
        range: Range<usize>,
        open_ctx: TokenWithContext,
        found: TokenOption,
    },
    /// expected open brace `{expected}`, found {found}
    ExpectedOpenBrace {
        expected: JsonChar,
        context: Option<TokenWithContext>,
        found: TokenOption,
    },

    // number
    /// expected digit following minus sign, found {1}
    ExpectedDigitFollowingMinus(Range<usize>, JsonCharOption),
    /// expected '-' or digit to start number, found {0}
    ExpectedMinusOrDigit(JsonCharOption),
    /// unexpected leading zero
    UnexpectedLeadingZero {
        initial: Range<usize>,
        extra: Range<usize>,
    },
    /// expected fraction digit following dot, found {maybe_c}
    ExpectedDigitAfterDot {
        number_range: Range<usize>,
        dot_range: Range<usize>,
        maybe_c: JsonCharOption,
    },
    /// expected +/- or digit after exponent indicator, found {maybe_c}
    ExpectedPlusOrMinusOrDigitAfterE {
        number_range: Range<usize>,
        e_range: Range<usize>,
        maybe_c: JsonCharOption,
    },
    /// expected digit after exponent indicator, found {maybe_c}
    ExpectedDigitAfterE {
        number_range: Range<usize>,
        exponent_range: Range<usize>,
        maybe_c: JsonCharOption,
    },

    // string
    /// unexpected unescaped control character `{0}` in string literal
    UnexpectedControlCharacterInString(JsonChar),
    /// expected closing quote
    ExpectedQuote {
        open_range: Range<usize>,
        string_range: Range<usize>,
    },
    /// expected hex digit {digit_idx} of 4 in escape, found {maybe_c}
    ExpectedHexDigit {
        quote_range: Range<usize>,
        slash_range: Range<usize>,
        u_range: Range<usize>,
        maybe_c: JsonCharOption,
        digit_idx: usize,
    },
    /** expected escapable sequence, found {maybe_c}.
    valid escapes are `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t` or `\uXXXX` (4 hex digits) */
    ExpectedEscape {
        maybe_c: JsonCharOption,
        slash_range: Range<usize>,
        string_range: Range<usize>,
        quote_range: Range<usize>,
    },

    // misc
    /// source did not contain valid utf8
    InvalidEncoding,
    /// unexpected character `{0}`. expected start of a json value
    UnexpectedCharacter(JsonChar),
    /// unexpected token {0} after json finished
    TokenAfterEnd(Token),
}

impl ErrorKind {
    pub fn expected_entry_or_closed_delimiter(
        open_ctx: TokenWithContext,
        found: TokenOption,
    ) -> Option<Self> {
        closing_delimiter_for_open(&open_ctx.token).map(|expected| {
            Self::ExpectedEntryOrClosedDelimiter {
                open_ctx,
                expected,
                found,
            }
        })
    }
}

fn closing_delimiter_for_open(token: &Token) -> Option<JsonChar> {
    match token {
        Token::OpenCurlyBrace => Some('}'.into()),
        Token::OpenSquareBracket => Some(']'.into()),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq, Display, Error)]
// box inner error for performance--a Rust enum is as large as the largest
// variant so happy path case becomes 100s of bytes otherwise
pub struct Error(pub Box<ErrorInner>);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Display, Error)]
/// {kind} at line {line} column {column}
pub struct ErrorInner {
    kind: ErrorKind,
    range: Range<usize>,
    /// 1 indexed line number
    line: usize,
    /// 1 indexed column number
    column: usize,
    source_text: String,
    source_name: String,
}

impl From<ErrorInner> for Error {
    fn from(value: ErrorInner) -> Self {
        Error(Box::new(value))
    }
}

impl Deref for Error {
    type Target = ErrorInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Error {
    pub fn new(kind: ErrorKind, range: Range<usize>, text: &str) -> Self {
        // TODO take this as a param or have some sort of context
        let source_name = "stdin".into();
        let (line, column) = get_line_and_column(text, range.clone());
        ErrorInner {
            kind,
            range,
            line,
            column,
            source_text: text.into(),
            source_name,
        }
        .into()
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
    pub fn from_maybe_json_char_with_context(
        f: impl Fn(JsonCharOption) -> ErrorKind,
        maybe_c: Option<CharWithContext>,
        text: &str,
    ) -> Self {
        if let Some(CharWithContext(r, c)) = maybe_c {
            Error::new(f(Some(c).into()), r, text)
        } else {
            Error::from_unterminated(f(None.into()), text)
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
