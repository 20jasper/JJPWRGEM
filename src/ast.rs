use crate::error::{Error, Result};
use crate::tokens::{Token, str_to_tokens};
use core::iter::Peekable;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Null,
    String(String),
    Object(HashMap<String, Value>),
    Boolean(bool),
}

impl TryFrom<Token> for Value {
    type Error = crate::Error;

    fn try_from(token: Token) -> std::result::Result<Self, Self::Error> {
        Ok(match token {
            Token::String(s) => Value::String(s),
            Token::Null => Value::Null,
            Token::Boolean(b) => Value::Boolean(b),
            _ => return Err(Error::Custom("token is not a valid value".to_owned())),
        })
    }
}

pub fn parse_str(json: &str) -> Result<Value> {
    let tokens = str_to_tokens(json)?;
    parse_tokens(&mut tokens.into_iter().peekable(), true)
}

pub fn parse_tokens(
    tokens: &mut Peekable<impl Iterator<Item = Token>>,
    fail_on_multiple_value: bool,
) -> Result<Value> {
    let peeked = if let Some(peeked) = tokens.peek() {
        peeked
    } else {
        return Err(Error::Empty);
    };
    let val = match peeked {
        Token::OpenCurlyBracket => parse_object(tokens, fail_on_multiple_value)?,
        Token::Null | Token::String(_) | Token::Boolean(_) => {
            let token = tokens.next().unwrap();
            token.try_into().expect("token should be valid json value")
        }
        invalid => return Err(Error::ExpectedValue(invalid.clone())),
    };

    if fail_on_multiple_value && let Some(token) = tokens.peek() {
        return Err(Error::TokenAfterEnd(token.clone()));
    }
    Ok(val)
}

enum ObjectState {
    Open,
    KeyOrEnd(HashMap<String, Value>),
    NextKeyOrEnd(HashMap<String, Value>),
    Key(HashMap<String, Value>),
    Colon {
        key: String,
        map: HashMap<String, Value>,
    },
    Value {
        key: String,
        map: HashMap<String, Value>,
    },
    End(HashMap<String, Value>),
}

fn parse_object(
    tokens: &mut Peekable<impl Iterator<Item = Token>>,
    fail_on_multiple_value: bool,
) -> Result<Value> {
    let mut state = ObjectState::Open;

    while let Some(peeked) = tokens.peek() {
        match state {
            ObjectState::Open => match tokens.next().unwrap() {
                Token::OpenCurlyBracket => state = ObjectState::KeyOrEnd(HashMap::new()),
                invalid => return Err(Error::ExpectedOpening(invalid)),
            },
            ObjectState::KeyOrEnd(map) => match tokens.next().unwrap() {
                Token::ClosedCurlyBracket => {
                    state = ObjectState::End(map);
                }
                Token::String(s) => {
                    state = ObjectState::Colon { key: s, map };
                }
                invalid => return Err(Error::ExpectedKeyOrClosing(invalid)),
            },
            ObjectState::Colon { map, key } => match tokens.next().unwrap() {
                Token::Colon => state = ObjectState::Value { map, key },
                invalid => return Err(Error::ExpectedColon(invalid)),
            },
            ObjectState::Value { mut map, key } => {
                let json_value = match peeked {
                    Token::OpenCurlyBracket
                    | Token::String(_)
                    | Token::Null
                    | Token::Boolean(_) => parse_tokens(tokens, false)?,
                    invalid => return Err(Error::ExpectedValue(invalid.clone())),
                };

                map.insert(key, json_value);
                state = ObjectState::NextKeyOrEnd(map);
            }
            ObjectState::NextKeyOrEnd(map) => match tokens.next().unwrap() {
                Token::ClosedCurlyBracket => {
                    state = ObjectState::End(map);
                }
                Token::Comma => {
                    state = ObjectState::Key(map);
                }
                invalid => return Err(Error::ExpectedCommaOrClosing(invalid.clone())),
            },
            ObjectState::Key(map) => match tokens.next().unwrap() {
                Token::String(key) => {
                    state = ObjectState::Colon { key, map };
                }
                invalid => return Err(Error::ExpectedKey(invalid)),
            },
            ObjectState::End(map) => {
                if fail_on_multiple_value {
                    return Err(Error::TokenAfterEnd(peeked.clone()));
                }
                return Ok(Value::Object(map));
            }
        }
    }

    if let ObjectState::End(map) = state {
        Ok(Value::Object(map))
    } else {
        Err(Error::Custom("unterminated something".into()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn kv_to_map(tuples: &[(&str, Value)]) -> Value {
        Value::Object(
            tuples
                .iter()
                .map(|(k, v)| ((*k).into(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn empty() {
        assert_eq!(parse_str("").unwrap_err(), Error::Empty);
    }

    #[test]
    fn empty_object() {
        assert_eq!(parse_str("{}").unwrap(), kv_to_map(&[]));
    }

    #[test]
    fn one_key_value_pair() {
        assert_eq!(
            parse_str(r#"{"hi":"bye"}"#).unwrap(),
            kv_to_map(&[("hi", Value::String("bye".into()))])
        );
    }

    #[test]
    fn key_with_braces() {
        assert_eq!(
            parse_str(r#"{"h{}{}i":"bye"}"#).unwrap(),
            kv_to_map(&[("h{}{}i", Value::String("bye".into()))])
        );
    }

    #[test]
    fn finished_object_then_another_char() {
        assert_eq!(
            parse_str("{}{").unwrap_err(),
            Error::TokenAfterEnd(Token::OpenCurlyBracket)
        );
    }

    #[test]
    fn multiple_keys() {
        assert_eq!(
            parse_str(
                r#"{
                "rust": "is a must",
                "name": "ferris" 
            }"#
            )
            .unwrap(),
            (kv_to_map(&[
                ("rust", Value::String("is a must".into())),
                ("name", Value::String("ferris".into())),
            ]))
        );
    }

    #[test]
    fn nested_object() {
        let nested = |val| kv_to_map(&[("rust", val)]);
        assert_eq!(
            parse_str(
                r#"
                {
                    "rust": {
                        "rust": {
                            "rust": {
                                "rust": "rust"
                            }   
                        }   
                    }
                }        
            "#
            )
            .unwrap(),
            nested(nested(nested(nested(Value::String("rust".into())))))
        );
    }

    #[rstest::rstest]
    #[case(r#"{"hi", "#, Error::ExpectedColon(Token::Comma))]
    #[case(r#"{"hi": , "#, Error::ExpectedValue(Token::Comma))]
    #[case(r#"}"#, Error::ExpectedValue(Token::ClosedCurlyBracket))]
    #[case(r#"{{"#, Error::ExpectedKeyOrClosing(Token::OpenCurlyBracket))]
    #[case(r#"{"hi": null null"#, Error::ExpectedCommaOrClosing(Token::Null))]
    #[case(r#"{"hi": null, }"#, Error::ExpectedKey(Token::ClosedCurlyBracket))]
    fn expected_error(#[case] json: &str, #[case] expected: Error) {
        assert_eq!(parse_str(json), Err(expected));
    }

    #[rstest_reuse::template]
    #[rstest::rstest]
    #[case("null", Value::Null)]
    #[case("true", Value::Boolean(true))]
    #[case("false", Value::Boolean(false))]
    #[case("\"burger\"", Value::String("burger".into()))]
    fn primitive_template(#[case] primitive: &str, #[case] expected: Value) {}

    #[rstest_reuse::apply(primitive_template)]
    fn primitive_object_value(#[case] primitive: &str, #[case] expected: Value) {
        assert_eq!(
            parse_str(&format!(
                r#"{{
                "rust": {primitive}
            }}"#
            ))
            .unwrap(),
            kv_to_map(&[("rust", expected)])
        )
    }

    #[rstest_reuse::apply(primitive_template)]
    fn primitives(#[case] json: &str, #[case] expected: Value) {
        assert_eq!(parse_str(json), Ok(expected));
    }
}
