use crate::error::Error;
use crate::{ErrorKind, Result};
use core::iter;
use core::ops::Range;
use core::{iter::Peekable, str::CharIndices};

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
        if c.is_whitespace() {
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
                        't' => Token::Boolean(true),
                        'f' => Token::Boolean(false),
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

pub fn build_str_while<'a>(
    start: usize,
    input: &'a str,
    chars: &mut Peekable<CharIndices<'a>>,
) -> Result<&'a str> {
    let mut end = start;

    while let Some((i, c)) = chars.next_if(|(_, c)| *c != '"') {
        end = i + c.len_utf8();
    }

    if !matches!(chars.next(), Some((_, '"'))) {
        return Err(ErrorKind::ExpectedQuote(None).into());
    }

    Ok(&input[start..end])
}

impl<I> From<I> for Token
where
    I: Into<String>,
{
    fn from(value: I) -> Self {
        Token::String(value.into())
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
                    token: "rust".into(),
                    range: 1..7
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 7..8
                },
                TokenWithContext {
                    token: "is a must".into(),
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
                    token: "rust".into(),
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

    #[rstest::rstest]
    #[case(r#"a"#, Error::new(ErrorKind::UnexpectedCharacter('a'), 0..1, "a"))]
    #[case(r#"n"#, Error::new(ErrorKind::UnexpectedCharacter('n'), 0..1, "n"))]
    #[case(r#""hi"#, ErrorKind::ExpectedQuote(None).into())]
    fn should_not_parse_invalid_syntax(#[case] json: &str, #[case] error: Error) {
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
                    token: "rust".into(),
                    range: 18..24
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 24..25
                },
                TokenWithContext {
                    token: "is a must".into(),
                    range: 26..37
                },
                TokenWithContext {
                    token: Token::Comma,
                    range: 37..38
                },
                TokenWithContext {
                    token: "name".into(),
                    range: 55..61
                },
                TokenWithContext {
                    token: Token::Colon,
                    range: 61..62
                },
                TokenWithContext {
                    token: "ferris".into(),
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
