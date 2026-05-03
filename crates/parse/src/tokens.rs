pub mod lexical;
mod number;
mod stream;
mod string;

use core::{fmt::Display, range::Range};

pub use stream::TokenStream;

use crate::tokens::lexical::{JsonByte, JsonChar};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
    OpenCurlyBrace,
    ClosedCurlyBrace,
    Colon,
    Comma,
    OpenSquareBracket,
    ClosedSquareBracket,
    String(&'a str),
    Mantissa(&'a str),
    Exponent(&'a str),
    Null,
    Boolean(bool),
}

impl<'a> Token<'a> {
    pub fn is_start_of_value(&self) -> bool {
        matches!(
            self,
            Token::OpenCurlyBrace
                | Token::OpenSquareBracket
                | Token::String(_)
                | Token::Null
                | Token::Boolean(_)
                | Token::Mantissa(_)
        )
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Token::String(_) | Token::Null | Token::Boolean(_) | Token::Mantissa(_)
        )
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Exponent(exponent) => write!(f, "`e{exponent}`"),
            Token::Mantissa(mantissa) => write!(f, "`{mantissa}`"),
            other => {
                let val = match other {
                    Token::OpenCurlyBrace => "{",
                    Token::ClosedCurlyBrace => "}",
                    Token::Colon => ":",
                    Token::Comma => ",",
                    Token::OpenSquareBracket => "[",
                    Token::ClosedSquareBracket => "]",
                    Token::String(x) => &format!("{x:?}"),
                    Token::Boolean(x) => &format!("{x:?}"),
                    Token::Null => NULL,
                    Token::Mantissa(_) | Token::Exponent(_) => unreachable!(),
                };
                write!(f, "`{val}`")
            }
        }
    }
}

const NO_SIGNIFICANT_CHARACTERS: &str = "no significant characters";
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TokenOption<'a>(pub(crate) Option<Token<'a>>);

impl Display for TokenOption<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match &self.0 {
            Some(x) => x.to_string(),
            None => NO_SIGNIFICANT_CHARACTERS.to_owned(),
        };
        write!(f, "{val}")
    }
}

impl<'a> From<Option<Token<'a>>> for TokenOption<'a> {
    fn from(value: Option<Token<'a>>) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct JsonCharOption(pub Option<JsonChar>);

impl Display for JsonCharOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match &self.0 {
            Some(x) => format!("`{x}`"),
            None => NO_SIGNIFICANT_CHARACTERS.to_owned(),
        };
        write!(f, "{val}")
    }
}

impl From<Option<JsonChar>> for JsonCharOption {
    fn from(value: Option<JsonChar>) -> Self {
        Self(value)
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TokenWithContext<'a> {
    pub token: Token<'a>,
    pub range: Range<usize>,
}

pub const NULL: &str = "null";
pub const FALSE: &str = "false";
pub const TRUE: &str = "true";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharWithContext(pub Range<usize>, pub JsonChar);
impl From<(usize, char)> for CharWithContext {
    fn from((i, c): (usize, char)) -> Self {
        Self(i..i + c.len_utf8(), c.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteWithContext(pub usize, pub JsonByte);
impl From<(usize, u8)> for ByteWithContext {
    fn from((i, b): (usize, u8)) -> Self {
        Self(i, b.into())
    }
}

impl ByteWithContext {
    pub fn range(&self) -> Range<usize> {
        self.0..self.0 + 1
    }

    pub fn as_token_with_context<'a>(&self) -> Option<TokenWithContext<'a>> {
        Some(TokenWithContext {
            token: self.1.as_token()?,
            range: self.range(),
        })
    }

    pub fn as_byte(&self) -> u8 {
        self.1.0
    }
}

impl From<bool> for Token<'_> {
    fn from(value: bool) -> Self {
        Token::Boolean(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, ErrorKind, Result};

    fn str_to_tokens<'a>(s: &'a str) -> Result<'a, Vec<TokenWithContext<'a>>> {
        stream::TokenStream::new(s).collect()
    }
    #[test]
    fn should_parse_single_key_object() {
        assert_eq!(
            str_to_tokens(r#"{"rust": "is a must"}"#).unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::String("rust"),
                    range: 1..7
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 7..8
                },
                TokenWithContext {
                    token: Token::String("is a must"),
                    range: 9..20
                },
                TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    range: 20..21
                }
            ]
        )
    }

    #[rstest_reuse::template]
    #[rstest::rstest]
    #[case("null", Token::Null)]
    #[case("true", Token::Boolean(true))]
    #[case("false", Token::Boolean(false))]
    #[case("\"burger\"", Token::String("burger"))]
    #[case(r#""\"burger\"""#, Token::String(r#"\"burger\""#))]
    fn primitive_template(#[case] json: &str, #[case] expected: Token) {}

    #[rstest_reuse::apply(primitive_template)]
    fn primitives(#[case] json: &str, #[case] expected: Token) {
        assert_eq!(
            str_to_tokens(json),
            Ok(vec![TokenWithContext {
                token: expected,
                range: 0..json.len()
            }])
        );
    }

    #[rstest_reuse::apply(primitive_template)]
    fn primitive_object_value(#[case] primitive: &str, #[case] expected: Token) {
        let json = format!(
            r#"{{
                "rust": {primitive}
            }}"#
        );
        assert_eq!(
            str_to_tokens(&json).unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::String("rust"),
                    range: 18..24
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 24..25
                },
                TokenWithContext {
                    token: expected,
                    range: 26..(26 + primitive.len())
                },
                TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    range: (json.len() - 1)..json.len()
                }
            ]
        )
    }

    fn json_to_json_and_error<'a>(
        json: &'static str,
        kind: ErrorKind<'a>,
        range: Option<Range<usize>>,
    ) -> (&'static str, Error<'a>) {
        let error = match range {
            Some(range) => Error::new(kind, range, json),
            None => Error::from_unterminated(kind, json),
        };
        (json, error)
    }

    #[rstest::rstest]
    #[case(json_to_json_and_error(
        "a",
        ErrorKind::UnexpectedCharacter('a'.into()),
        Some(0..1)
    ))]
    #[case(json_to_json_and_error(
        "n",
        ErrorKind::UnexpectedCharacter('n'.into()),
        Some(0..1)
    ))]
    #[case(json_to_json_and_error(
        r#""
    
    ""#,
        ErrorKind::UnexpectedControlCharacterInString('\n'.into()),
        Some(1..2)
    ))]
    fn should_not_parse_invalid_syntax(#[case] (json, error): (&str, Error)) {
        assert_eq!(str_to_tokens(json), Err(error));
    }

    #[test]
    fn multiple_keys() {
        assert_eq!(
            str_to_tokens(
                r#"{
                "rust": "is a must",
                "name": "ferris"
            }"#
            )
            .unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::String("rust"),
                    range: 18..24
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 24..25
                },
                TokenWithContext {
                    token: Token::String("is a must"),
                    range: 26..37
                },
                TokenWithContext {
                    token: Token::Comma,
                    range: 37..38
                },
                TokenWithContext {
                    token: Token::String("name"),
                    range: 55..61
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 61..62
                },
                TokenWithContext {
                    token: Token::String("ferris"),
                    range: 63..71
                },
                TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    range: 84..85
                }
            ]
        );
    }

    #[test]
    fn array_brackets() {
        assert_eq!(
            str_to_tokens("[]").unwrap(),
            [
                TokenWithContext {
                    token: Token::OpenSquareBracket,
                    range: 0..1
                },
                TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: 1..2
                }
            ]
        )
    }
}
