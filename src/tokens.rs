use crate::error::Error;
use crate::{ErrorKind, Result};
use core::iter;
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

pub const NULL: &str = "null";
pub const FALSE: &str = "false";
pub const TRUE: &str = "true";

pub fn str_to_tokens(s: &str) -> Result<Vec<Token>> {
    let mut chars = s.char_indices().peekable();

    let mut res = vec![];

    while let Some((i, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        let val = match c {
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
        res.push(val);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_single_key_object() {
        assert_eq!(
            str_to_tokens(r#"{"rust": "is a must"}"#).unwrap(),
            [
                Token::OpenCurlyBrace,
                Token::String("rust".into()),
                Token::Colon,
                Token::String("is a must".into()),
                Token::ClosedCurlyBrace,
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
        assert_eq!(str_to_tokens(json), Ok(vec![expected]));
    }

    #[rstest_reuse::apply(primitive_template)]
    fn primitive_object_value(#[case] primitive: &str, #[case] expected: Token) {
        assert_eq!(
            str_to_tokens(&format!(
                r#"{{
                "rust": {primitive}
            }}"#
            ))
            .unwrap(),
            [
                Token::OpenCurlyBrace,
                Token::String("rust".into()),
                Token::Colon,
                expected,
                Token::ClosedCurlyBrace,
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
                Token::OpenCurlyBrace,
                Token::String("rust".into()),
                Token::Colon,
                Token::String("is a must".into()),
                Token::Comma,
                Token::String("name".into()),
                Token::Colon,
                Token::String("ferris".into()),
                Token::ClosedCurlyBrace,
            ]
        );
    }
}
