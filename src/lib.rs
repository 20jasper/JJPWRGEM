mod error;
mod tokens;

use crate::tokens::{str_to_tokens, Token};
use error::{Error, Result};
use std::collections::HashMap;

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
pub enum Value {
    Null,
    String(String),
    Object(HashMap<String, Value>),
}

pub fn parse(json: &str) -> Result<Value> {
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

    Ok(Value::Object(map))
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
        assert_eq!(parse("{}").unwrap(), Value::Object(HashMap::new()));
    }

    #[test]
    fn one_key_value_pair() {
        assert_eq!(
            parse(r#"{"hi":"bye"}"#).unwrap(),
            Value::Object(kv_to_map(&[("hi", Value::String("bye".into()))]))
        );
    }

    #[test]
    fn key_with_braces() {
        assert_eq!(
            parse(r#"{"h{}{}i":"bye"}"#).unwrap(),
            Value::Object(kv_to_map(&[("h{}{}i", Value::String("bye".into()))]))
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
            Value::Object(kv_to_map(&[
                ("rust", Value::String("is a must".into())),
                ("name", Value::String("ferris".into())),
            ]))
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
            Value::Object(kv_to_map(&[("rust", Value::Null)]))
        )
    }
}
