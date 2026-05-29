use core::{ops::Deref, range::Range};

use displaydoc::Display;

use crate::tokens::{
    CharWithContext, ErrorToken, JsonCharOption, Token, TokenOption, TokenWithContext,
    lexical::{JsonChar, trim_end_whitespace},
};

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

    // mantissa
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

    // exponent
    /// expected +/- or digit after exponent indicator, found {maybe_c}
    ExpectedPlusOrMinusOrDigitAfterE {
        e_range: Range<usize>,
        maybe_c: JsonCharOption,
    },
    /// expected digit after exponent indicator, found {maybe_c}
    ExpectedDigitAfterE {
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
    /// {_0}
    InvalidEncoding(bytes2chars::ErrorKind),
    /// unexpected character `{0}`. expected start of a json value
    UnexpectedCharacter(JsonChar),
    /// unexpected token {0} after json finished
    TokenAfterEnd(ErrorToken),
    /// json is nested too deeply (max depth: {0})
    NestingTooDeep(usize),
}

impl ErrorKind {
    pub(crate) fn expected_entry_or_closed_delimiter(
        open_ctx: TokenWithContext,
        found: TokenOption,
    ) -> Option<Self> {
        closing_delimiter_for_open(open_ctx.token).map(|expected| {
            Self::ExpectedEntryOrClosedDelimiter {
                open_ctx,
                expected,
                found,
            }
        })
    }
}

fn closing_delimiter_for_open(token: Token) -> Option<JsonChar> {
    match token {
        Token::OpenCurlyBrace => Some('}'.into()),
        Token::OpenSquareBracket => Some(']'.into()),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq, Display, Clone)]
// box inner error for performance--a Rust enum is as large as the largest
// variant so happy path case becomes 100s of bytes otherwise
/// {0}
pub struct Error(pub(crate) Box<ErrorInner>);

impl std::error::Error for Error {}

#[derive(Debug, PartialEq, Eq, Display, Clone)]
/// {kind}
pub struct ErrorInner {
    pub(crate) kind: ErrorKind,
    pub(crate) range: Range<usize>,
    pub(crate) source_text: String,
    pub(crate) source_name: String,
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
    pub fn message(&self) -> String {
        self.to_string()
    }

    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    /// byte range
    pub fn range(&self) -> &Range<usize> {
        &self.0.range
    }

    pub(crate) fn new(kind: ErrorKind, range: Range<usize>, text: &str) -> Self {
        ErrorInner {
            kind,
            range,
            source_text: text.into(),
            source_name: "stdin".into(),
        }
        .into()
    }

    pub(crate) fn from_unterminated(kind: ErrorKind, text: &str) -> Self {
        let trimmed = trim_end_whitespace(text);
        let last_char_start = trimmed.char_indices().next_back().map_or(0, |(i, _)| i);
        Self::new(kind, last_char_start..trimmed.len(), text)
    }

    /// # Panics
    /// if bytes are valid at the location reported by [std::str::Utf8Error]
    pub fn from_utf8_error_slice(e: std::str::Utf8Error, bytes: &[u8]) -> Error {
        use bytes2chars::Utf8CharIndices;
        const LOSSY_BYTE_LENGTH: usize = '\u{FFFD}'.len_utf8();
        let b2c_err =
            Utf8CharIndices::new(bytes[e.valid_up_to()..].iter().copied(), e.valid_up_to())
                .next()
                .and_then(|r| r.err())
                .expect("a Utf8Error was returned so this must be an error");

        ErrorInner {
            kind: ErrorKind::InvalidEncoding(b2c_err.kind),
            range: e.valid_up_to()..e.valid_up_to() + LOSSY_BYTE_LENGTH,
            source_text: String::from_utf8_lossy(bytes).into_owned(),
            source_name: "stdin".into(),
        }
        .into()
    }

    pub(crate) fn from_maybe_token_with_context<F>(
        f: F,
        maybe_token: Option<TokenWithContext>,
        text: &str,
    ) -> Self
    where
        F: Fn(TokenOption) -> ErrorKind,
    {
        if let Some(twc) = maybe_token {
            Error::new(
                f(TokenOption(Some(ErrorToken::new(
                    twc.token, twc.range, text,
                )))),
                twc.range,
                text,
            )
        } else {
            Error::from_unterminated(f(None.into()), text)
        }
    }

    pub(crate) fn from_maybe_json_char_with_context<F>(
        f: F,
        maybe_c: Option<CharWithContext>,
        text: &str,
    ) -> Self
    where
        F: Fn(JsonCharOption) -> ErrorKind,
    {
        if let Some(CharWithContext(r, c)) = maybe_c {
            Error::new(f(Some(c).into()), r, text)
        } else {
            Error::from_unterminated(f(None.into()), text)
        }
    }
}
