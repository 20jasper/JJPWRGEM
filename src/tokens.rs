use core::iter;

use crate::string::build_str_while;
use crate::{Error, Result};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    OpenCurlyBracket,
    ClosedCurlyBracket,
    Colon,
    Comma,
    String(String),
    Null,
}

pub fn str_to_tokens(s: &str) -> Result<Vec<Token>> {
    let mut chars = s.char_indices().peekable();

    let mut res = vec![];

    while let Some((i, c)) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        let val = match c {
            '{' => Token::OpenCurlyBracket,
            '}' => Token::ClosedCurlyBracket,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '"' => Token::String(build_str_while(i + 1, s, &mut chars).into()),
            'n' => {
                let actual = iter::once('n').chain(chars.by_ref().take(3).map(|(_, c)| c));
                let expected = "null".chars();

                if actual.eq(expected) {
                    Token::Null
                } else {
                    return Err(Error::UnexpectedCharacter('n'));
                }
            }
            invalid => return Err(Error::UnexpectedCharacter(invalid)),
        };
        res.push(val);
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_single_key_object() {
        assert_eq!(
            str_to_tokens(r#"{"rust": "is a must"}"#).unwrap(),
            [
                Token::OpenCurlyBracket,
                Token::String("rust".into()),
                Token::Colon,
                Token::String("is a must".into()),
                Token::ClosedCurlyBracket,
            ]
        )
    }

    #[test]
    fn null() {
        assert_eq!(str_to_tokens(r#"null"#), Ok(vec![Token::Null]));
    }

    #[test]
    fn should_not_parse_invalid_syntax() {
        assert_eq!(str_to_tokens(r#"a"#), Err(Error::UnexpectedCharacter('a')));
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
                Token::OpenCurlyBracket,
                Token::String("rust".into()),
                Token::Colon,
                Token::String("is a must".into()),
                Token::Comma,
                Token::String("name".into()),
                Token::Colon,
                Token::String("ferris".into()),
                Token::ClosedCurlyBracket,
            ]
        );
    }
}
