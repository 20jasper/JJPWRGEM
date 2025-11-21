pub mod lexical;

pub use lexical::{CONTROL_RANGE, trim_end_whitespace};

use self::lexical::{escape_char_for_json_string, is_whitespace};
use crate::{Error, ErrorKind, Result};
use core::fmt::Display;
use core::iter;
use core::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenCurlyBrace,
    ClosedCurlyBrace,
    Colon,
    Comma,
    String(String),
    Null,
    Boolean(bool),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Token::OpenCurlyBrace => "{",
            Token::ClosedCurlyBrace => "}",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::String(x) => &format!("{x:?}"),
            Token::Boolean(x) => &format!("{x:?}"),
            Token::Null => NULL,
        };
        write!(f, "`{val}`")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenOption(pub(crate) Option<Token>);

impl Display for TokenOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match &self.0 {
            Some(x) => x.to_string(),
            None => "no significant characters".to_owned(),
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
pub struct TokenWithContext {
    pub token: Token,
    pub range: Range<usize>,
}

pub const NULL: &str = "null";
pub const FALSE: &str = "false";
pub const TRUE: &str = "true";

pub fn str_to_tokens(s: &str) -> Result<Vec<TokenWithContext>> {
    let mut chars = s.char_indices().peekable();

    let mut res = vec![];

    while let Some((i, c)) = chars.next() {
        if is_whitespace(c) {
            continue;
        }
        let token = match c {
            '{' => Token::OpenCurlyBrace,
            '}' => Token::ClosedCurlyBrace,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '"' => Token::String(build_str_while(i + 1, s, &mut chars)?.into()),
            'n' | 't' | 'f' => {
                let expected = match c {
                    'n' => NULL,
                    't' => TRUE,
                    'f' => FALSE,
                    _ => unreachable!("{c} is not able to be reached"),
                };
                let actual =
                    iter::once(c).chain(chars.by_ref().take(expected.len() - 1).map(|(_, c)| c));

                if actual.eq(expected.chars()) {
                    match c {
                        'n' => Token::Null,
                        't' => true.into(),
                        'f' => false.into(),
                        _ => unreachable!("{c} is not able to be reached"),
                    }
                } else {
                    return Err(Error::new(
                        ErrorKind::UnexpectedCharacter(c),
                        i..(i + c.len_utf8()),
                        s,
                    ));
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::UnexpectedCharacter(c),
                    i..(i + c.len_utf8()),
                    s,
                ));
            }
        };
        let start = i;
        let end = *chars.peek().map(|(i, _)| i).unwrap_or(&s.len());
        res.push(TokenWithContext {
            token,
            range: start..end,
        });
    }

    Ok(res)
}

fn build_str_while<'a>(
    start: usize,
    input: &'a str,
    chars: &mut core::iter::Peekable<core::str::CharIndices<'a>>,
) -> Result<&'a str> {
    let mut escape = false;
    while let Some((_, c)) =
        chars.next_if(|(_, c)| (*c != '"' && !CONTROL_RANGE.contains(c)) || escape)
    {
        escape = c == '\\' && !escape;
    }

    if let Some((end, c)) = chars.next() {
        if !CONTROL_RANGE.contains(&c) {
            Ok(&input[start..end])
        } else {
            Err(Error::new(
                ErrorKind::UnexpectedControlCharacterInString(escape_char_for_json_string(c)),
                end..end + c.len_utf8(),
                input,
            ))
        }
    } else {
        Err(Error::from_unterminated(ErrorKind::ExpectedQuote, input))
    }
}

impl From<bool> for Token {
    fn from(value: bool) -> Self {
        Token::Boolean(value)
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
        ErrorKind::UnexpectedCharacter('a'),
        Some(0..1)
    ))]
    #[case(json_to_json_and_error(
        "n",
        ErrorKind::UnexpectedCharacter('n'),
        Some(0..1)
    ))]
    #[case(json_to_json_and_error(r#""hi"#, ErrorKind::ExpectedQuote, None))]
    #[case(json_to_json_and_error(
        r#""
    
    ""#,
        ErrorKind::UnexpectedControlCharacterInString("\\n".to_string()),
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
}
