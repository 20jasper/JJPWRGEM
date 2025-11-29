pub mod lexical;
mod number;

use crate::{
    Error, ErrorKind, Result,
    tokens::{lexical::JsonChar, number::parse_num, string::parse_string},
};
use core::fmt::Display;
use core::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenCurlyBrace,
    ClosedCurlyBrace,
    Colon,
    Comma,
    OpenSquareBracket,
    ClosedSquareBracket,
    String(String),
    Number(String),
    Null,
    Boolean(bool),
}

impl Token {
    pub fn is_start_of_value(&self) -> bool {
        matches!(
            self,
            Token::OpenCurlyBrace
                | Token::OpenSquareBracket
                | Token::String(_)
                | Token::Null
                | Token::Boolean(_)
                | Token::Number(_)
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
            Token::String(x) => &format!("{x:?}"),
            Token::Number(x) => &x.to_string(),
            Token::Boolean(x) => &format!("{x:?}"),
            Token::Null => NULL,
        };
        write!(f, "`{val}`")
    }
}

const NO_SIGNIFICANT_CHARACTERS: &str = "no significant characters";
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenOption(pub(crate) Option<Token>);

impl Display for TokenOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match &self.0 {
            Some(x) => x.to_string(),
            None => NO_SIGNIFICANT_CHARACTERS.to_owned(),
        };
        write!(f, "{val}")
    }
}

impl From<Option<Token>> for TokenOption {
    fn from(value: Option<Token>) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenWithContext {
    pub token: Token,
    pub range: Range<usize>,
}

pub const NULL: &str = "null";
pub const FALSE: &str = "false";
pub const TRUE: &str = "true";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharWithContext(pub Range<usize>, pub JsonChar);
impl From<(usize, char)> for CharWithContext {
    fn from((i, c): (usize, char)) -> Self {
        Self(i..i + c.len_utf8(), c.into())
    }
}

impl CharWithContext {
    fn as_token_with_context(&self) -> Option<TokenWithContext> {
        match self {
            CharWithContext(range, JsonChar(c)) => {
                let token = match c {
                    '{' => Token::OpenCurlyBrace,
                    '}' => Token::ClosedCurlyBrace,
                    ':' => Token::Colon,
                    ',' => Token::Comma,
                    '[' => Token::OpenSquareBracket,
                    ']' => Token::ClosedSquareBracket,
                    _ => return None,
                };
                Some(TokenWithContext {
                    token,
                    range: range.clone(),
                })
            }
        }
    }

    fn as_json_char(&self) -> JsonChar {
        self.1
    }
    fn as_char(&self) -> char {
        self.as_json_char().0
    }
}

pub fn str_to_tokens(s: &str) -> Result<Vec<TokenWithContext>> {
    let mut chars = s
        .char_indices()
        .map(Into::<CharWithContext>::into)
        .peekable();

    let mut res = vec![];

    while let Some(ctx) = chars.peek().cloned() {
        let CharWithContext(r, JsonChar(c)) = ctx.clone();
        if ctx.as_json_char().is_whitespace() {
            chars.next();
            continue;
        }
        if let Some(tok) = ctx.as_token_with_context() {
            res.push(tok);
            chars.next();
            continue;
        }
        let token = match c {
            '"' => parse_string(s, &mut chars)?,
            '0'..='9' | '-' => parse_num(s, &mut chars)?,
            'n' | 't' | 'f' => {
                let expected = match c {
                    'n' => NULL,
                    't' => TRUE,
                    'f' => FALSE,
                    _ => unreachable!("{c} is not able to be reached"),
                };
                let actual = chars.by_ref().take(expected.len()).map(|c| c.as_char());

                if actual.eq(expected.chars()) {
                    let token = match c {
                        'n' => Token::Null,
                        't' => true.into(),
                        'f' => false.into(),
                        _ => unreachable!("{c} is not able to be reached"),
                    };
                    let start = r.start;
                    let end = *chars
                        .peek()
                        .map(|CharWithContext(r, _)| &r.start)
                        .unwrap_or(&s.len());
                    TokenWithContext {
                        token,
                        range: start..end,
                    }
                } else {
                    return Err(Error::new(
                        ErrorKind::UnexpectedCharacter(c.into()),
                        r.clone(),
                        s,
                    ));
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedCharacter(c.into()),
                    r.clone(),
                    s,
                ));
            }
        };
        res.push(token);
    }

    Ok(res)
}

mod string {
    use itertools::Itertools;

    use crate::{
        Error, ErrorKind, Result,
        tokens::{CharWithContext, JsonChar, Token, TokenWithContext},
    };
    use core::ops::Range;
    use std::iter::Peekable;

    enum StringState {
        Open,
        // TODO track last escaped u escapes
        CharOrEscapeOrEnd {
            string_range: Range<usize>,
            quote_range: Range<usize>,
        },
        Escape {
            string_range: Range<usize>,
            quote_range: Range<usize>,
        },
        End(TokenWithContext),
    }

    impl StringState {
        fn process(
            self,
            chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
            input: &str,
        ) -> Result<Self> {
            let res = match self {
                StringState::Open => {
                    let Some(CharWithContext(starting_quote, JsonChar('"'))) = chars.next() else {
                        unreachable!("must start with a quote");
                    };

                    StringState::CharOrEscapeOrEnd {
                        string_range: starting_quote.clone(),
                        quote_range: starting_quote.clone(),
                    }
                }
                StringState::CharOrEscapeOrEnd {
                    string_range,
                    quote_range,
                } => match chars.next() {
                    Some(CharWithContext(r, JsonChar('\\'))) => StringState::Escape {
                        string_range: string_range.start..r.end,
                        quote_range,
                    },
                    Some(CharWithContext(r, JsonChar('"'))) => StringState::End(TokenWithContext {
                        token: Token::String(input[quote_range.end..r.start].into()),
                        range: string_range.start..r.end,
                    }),
                    Some(CharWithContext(r, c)) if c.is_control() => {
                        return Err(Error::new(
                            ErrorKind::UnexpectedControlCharacterInString(c),
                            r,
                            input,
                        ));
                    }
                    Some(CharWithContext(r, _)) => StringState::CharOrEscapeOrEnd {
                        string_range: string_range.start..r.end,
                        quote_range,
                    },
                    None => return Err(Error::from_unterminated(ErrorKind::ExpectedQuote, input)),
                },
                StringState::Escape {
                    string_range,
                    quote_range,
                } => match chars.next() {
                    Some(CharWithContext(r, c)) if c.can_be_escaped_directly() => {
                        StringState::CharOrEscapeOrEnd {
                            string_range: string_range.start..r.end,
                            quote_range,
                        }
                    }
                    Some(CharWithContext(r, JsonChar('u'))) => {
                        todo!()
                    }
                    _ => todo!(),
                },
                StringState::End(_) => self,
            };

            Ok(res)
        }
    }

    pub fn parse_string(
        input: &str,
        chars: &mut Peekable<impl Iterator<Item = CharWithContext>>,
    ) -> Result<TokenWithContext> {
        let mut state = StringState::Open;

        loop {
            state = state.process(chars, input)?;
            if let StringState::End(tok) = state {
                break Ok(tok);
            }
        }
    }
}

impl From<bool> for Token {
    fn from(value: bool) -> Self {
        Token::Boolean(value)
    }
}
// TODO macro for every num type
impl From<usize> for Token {
    fn from(value: usize) -> Self {
        Token::Number(value.to_string())
    }
}
impl From<i32> for Token {
    fn from(value: i32) -> Self {
        Token::Number(value.to_string())
    }
}
impl From<f64> for Token {
    fn from(value: f64) -> Self {
        Token::Number(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    token: Token::String("rust".into()),
                    range: 1..7
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 7..8
                },
                TokenWithContext {
                    token: Token::String("is a must".into()),
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
    #[case("\"burger\"", Token::String("burger".into()))]
    #[case(r#""\"burger\"""#, Token::String(r#"\"burger\""#.into()))]
    #[case(r#"0"#, 0.into())]
    #[case(r#"12389"#, 12389.into())]
    #[case(r#"-12389"#, (-12389).into())]
    // #[case(r#"5.8888"#, 5.888.into())]
    #[case(r#"-0"#, Token::Number("-0".into()))]
    // #[case(r#"-1e5"#, Token::Number("-1e5".into()))]
    // #[case(r#"-1.48e50"#, Token::Number("-1.48e50".into()))]
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
                    token: Token::String("rust".into()),
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
    #[case(json_to_json_and_error(r#""hi"#, ErrorKind::ExpectedQuote, None))]
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
                    token: Token::String("rust".into()),
                    range: 18..24
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 24..25
                },
                TokenWithContext {
                    token: Token::String("is a must".into()),
                    range: 26..37
                },
                TokenWithContext {
                    token: Token::Comma,
                    range: 37..38
                },
                TokenWithContext {
                    token: Token::String("name".into()),
                    range: 55..61
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 61..62
                },
                TokenWithContext {
                    token: Token::String("ferris".into()),
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
