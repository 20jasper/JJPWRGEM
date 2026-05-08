pub mod lexical;
mod number;
mod stream;
mod string;

use core::{fmt::Display, iter::Peekable, range::Range};

pub use stream::TokenStream;

use crate::tokens::lexical::{JsonByte, JsonChar};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    OpenCurlyBrace,
    ClosedCurlyBrace,
    Colon,
    Comma,
    OpenSquareBracket,
    ClosedSquareBracket,
    String,
    Mantissa,
    Exponent,
    Null,
    True,
    False,
}

impl Token {
    pub fn is_start_of_value(&self) -> bool {
        matches!(
            self,
            Token::OpenCurlyBrace
                | Token::OpenSquareBracket
                | Token::String
                | Token::Null
                | Token::True
                | Token::False
                | Token::Mantissa
        )
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Token::String | Token::Null | Token::True | Token::False | Token::Mantissa
        )
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Token::OpenCurlyBrace => "{",
            Token::ClosedCurlyBrace => "}",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::OpenSquareBracket => "[",
            Token::ClosedSquareBracket => "]",
            Token::String => "string",
            Token::Mantissa => "mantissa",
            Token::Exponent => "exponent",
            Token::Null => NULL,
            Token::True => TRUE,
            Token::False => FALSE,
        };
        write!(f, "`{val}`")
    }
}

const NO_SIGNIFICANT_CHARACTERS: &str = "no significant characters";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorToken {
    pub(crate) tag: Token,
    content: Box<str>,
}

impl ErrorToken {
    pub fn new(tag: Token, range: Range<usize>, source: &str) -> Self {
        Self {
            tag,
            content: source[range.start..range.end].into(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Display for ErrorToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag {
            Token::String | Token::Mantissa => write!(f, "`{}`", self.content),
            Token::Exponent => write!(f, "`e{}`", self.content),
            t => write!(f, "{t}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenOption(pub(crate) Option<ErrorToken>);

impl Display for TokenOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(t) => write!(f, "{t}"),
            None => write!(f, "{NO_SIGNIFICANT_CHARACTERS}"),
        }
    }
}

impl From<Option<Token>> for TokenOption {
    fn from(value: Option<Token>) -> Self {
        Self(value.map(|t| ErrorToken {
            tag: t,
            content: "".into(),
        }))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenWithContext {
    pub token: Token,
    pub range: Range<usize>,
}

impl TokenWithContext {
    pub fn new(token: Token, range: Range<usize>) -> Self {
        Self { token, range }
    }

    pub fn content_range(&self) -> Range<usize> {
        match self.token {
            Token::String => self.range.start + 1..self.range.end - 1,
            _ => self.range,
        }
    }
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

    pub fn as_token_with_context(&self) -> Option<TokenWithContext> {
        Some(TokenWithContext::new(self.1.as_token()?, self.range()))
    }

    pub fn as_byte(&self) -> u8 {
        self.1.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BytesWithContext<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> BytesWithContext<'a> {
    pub(crate) fn new(input: &'a str, pos: usize) -> Self {
        Self { input, pos }
    }
}

impl Iterator for BytesWithContext<'_> {
    type Item = ByteWithContext;

    fn next(&mut self) -> Option<Self::Item> {
        self.input.as_bytes().get(self.pos).copied().map(|byte| {
            let start = self.pos;
            self.pos += 1;
            (start, byte).into()
        })
    }
}

pub(crate) fn current_byte_pos(
    bytes: &mut Peekable<impl Iterator<Item = ByteWithContext>>,
    input: &str,
) -> usize {
    bytes
        .peek()
        .map(|ByteWithContext(start, _)| *start)
        .unwrap_or(input.len())
}

impl From<bool> for Token {
    fn from(value: bool) -> Self {
        if value { Token::True } else { Token::False }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, ErrorKind, Result};

    fn str_to_tokens(s: &str) -> Result<Vec<TokenWithContext>> {
        stream::TokenStream::new(s).collect()
    }

    fn ctx(token: Token, range: Range<usize>) -> TokenWithContext {
        TokenWithContext::new(token, range)
    }

    #[test]
    fn should_parse_single_key_object() {
        assert_eq!(
            str_to_tokens(r#"{"rust": "is a must"}"#).unwrap(),
            [
                ctx(Token::OpenCurlyBrace, 0..1),
                ctx(Token::String, 1..7),
                ctx(Token::Colon, 7..8),
                ctx(Token::String, 9..20),
                ctx(Token::ClosedCurlyBrace, 20..21),
            ]
        )
    }

    #[rstest_reuse::template]
    #[rstest::rstest]
    #[case("null", Token::Null)]
    #[case("true", Token::True)]
    #[case("false", Token::False)]
    #[case("\"burger\"", Token::String)]
    #[case(r#""\"burger\"""#, Token::String)]
    fn primitive_template(#[case] json: &str, #[case] expected: Token) {}

    #[rstest_reuse::apply(primitive_template)]
    fn primitives(#[case] json: &str, #[case] expected: Token) {
        assert_eq!(str_to_tokens(json), Ok(vec![ctx(expected, 0..json.len())]));
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
                ctx(Token::OpenCurlyBrace, 0..1),
                ctx(Token::String, 18..24),
                ctx(Token::Colon, 24..25),
                ctx(expected, 26..(26 + primitive.len())),
                ctx(Token::ClosedCurlyBrace, (json.len() - 1)..json.len()),
            ]
        )
    }

    fn json_to_json_and_error(
        json: &'static str,
        kind: ErrorKind,
        range: Option<Range<usize>>,
    ) -> (&'static str, Error) {
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
                ctx(Token::OpenCurlyBrace, 0..1),
                ctx(Token::String, 18..24),
                ctx(Token::Colon, 24..25),
                ctx(Token::String, 26..37),
                ctx(Token::Comma, 37..38),
                ctx(Token::String, 55..61),
                ctx(Token::Colon, 61..62),
                ctx(Token::String, 63..71),
                ctx(Token::ClosedCurlyBrace, 84..85),
            ]
        );
    }

    #[test]
    fn array_brackets() {
        assert_eq!(
            str_to_tokens("[]").unwrap(),
            [
                ctx(Token::OpenSquareBracket, 0..1),
                ctx(Token::ClosedSquareBracket, 1..2),
            ]
        )
    }

    #[test]
    fn token_with_context_size() {
        assert_eq!(core::mem::size_of::<Token>(), 1);
        assert_eq!(core::mem::size_of::<TokenWithContext>(), 24);
    }
}
