use std::{collections::HashMap, intrinsics::unreachable};

mod string {
    use core::{iter::Peekable, str::CharIndices};

    pub fn build_str_while<'a>(
        start: usize,
        input: &'a str,
        chars: &mut Peekable<CharIndices<'a>>,
    ) -> &'a str {
        let mut end = start;

        while let Some((i, c)) = chars.next_if(|(_, c)| *c != '"') {
            end = i + c.len_utf8();
        }
        chars.next();

        &input[start..end]
    }
}

mod tokens {
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
}

mod error;

use error::{Error, Result};

use crate::tokens::{str_to_tokens, Token};

enum State {
    Init,
    Object,
    NextObjectKeyOrFinish,
    NextObjectKey,
    End,
    Key,
    Value,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Value {
    Null,
    String(String),
}

pub fn parse(json: &str) -> Result<HashMap<String, Value>> {
    let tokens = str_to_tokens(json)?;

    let mut state = State::Init;

    if tokens.is_empty() {
        return Err(Error::Empty);
    }

    let mut map = HashMap::new();
    let mut key = None::<String>;

    for token in tokens {
        match state {
            State::Init => match token {
                Token::OpenCurlyBracket => {
                    state = State::Object;
                }
                Token::ClosedCurlyBracket => return Err(Error::Unmatched(token)),
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Object => match token {
                Token::ClosedCurlyBracket => {
                    state = State::End;
                }
                Token::String(s) => {
                    key = Some(s);
                    state = State::Key;
                }
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Key => match token {
                Token::Colon => state = State::Value,
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Value => match token {
                Token::String(_) | Token::Null => {
                    let val = match token {
                        Token::String(s) => Value::String(s),
                        Token::Null => Value::Null,
                        _ => unreachable!("{token:?} should not be reachable"),
                    };
                    map.insert(key.take().expect("key should have been found"), val);
                    state = State::NextObjectKeyOrFinish;
                }
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::NextObjectKeyOrFinish => match token {
                Token::ClosedCurlyBracket => {
                    state = State::End;
                }
                Token::Comma => {
                    state = State::NextObjectKey;
                }
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::NextObjectKey => match token {
                Token::String(s) => {
                    key = Some(s);
                    state = State::Key;
                }
                _ => return Err(Error::ExpectedKey),
            },
            State::End => return Err(Error::TokenAfterEnd(token)),
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv_to_map(tuples: &[(&str, Value)]) -> HashMap<String, Value> {
        tuples
            .iter()
            .map(|(k, v)| ((*k).into(), v.clone()))
            .collect()
    }

    #[test]
    fn empty() {
        assert_eq!(parse("").unwrap_err(), Error::Empty);
    }

    #[test]
    fn unmatched() {
        assert_eq!(
            parse("}").unwrap_err(),
            Error::Unmatched(Token::ClosedCurlyBracket)
        );
    }

    #[test]
    fn empty_object() {
        assert_eq!(parse("{}").unwrap(), HashMap::new());
    }

    #[test]
    fn one_key_value_pair() {
        assert_eq!(
            parse(r#"{"hi":"bye"}"#).unwrap(),
            kv_to_map(&[("hi", Value::String("bye".into()))])
        );
    }

    #[test]
    fn key_with_braces() {
        assert_eq!(
            parse(r#"{"h{}{}i":"bye"}"#).unwrap(),
            kv_to_map(&[("h{}{}i", Value::String("bye".into()))])
        );
    }

    #[test]
    fn finished_object_then_another_char() {
        assert_eq!(
            parse("{}{").unwrap_err(),
            Error::TokenAfterEnd(Token::OpenCurlyBracket)
        );
    }

    #[test]
    fn multiple_keys() {
        assert_eq!(
            parse(
                r#"{
                "rust": "is a must",
                "name": "ferris" 
            }"#
            )
            .unwrap(),
            kv_to_map(&[
                ("rust", Value::String("is a must".into())),
                ("name", Value::String("ferris".into())),
            ])
        );
    }

    #[test]
    fn trailing_commas_not_allowed() {
        assert_eq!(
            parse(
                r#"{
                "rust": "is a must",
            }"#
            )
            .unwrap_err(),
            Error::ExpectedKey
        );
    }

    #[test]
    fn null_object_value() {
        assert_eq!(
            parse(
                r#"{
                "rust": null
            }"#
            )
            .unwrap(),
            kv_to_map(&[("rust", Value::Null)])
        )
    }
}
