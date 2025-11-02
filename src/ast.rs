use crate::error::{Error, Result};
use crate::tokens::{str_to_tokens, Token};
use std::collections::HashMap;

enum State {
    Init,
    Object,
    NextObjectKeyOrFinish,
    NextObjectKey,
    End,
    Key(String),
    Value { key: String },
}

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

pub fn parse(json: &str) -> Result<Value> {
    let tokens = str_to_tokens(json)?;

    let mut state = State::Init;

    let mut val = None::<Value>;

    for token in tokens {
        match state {
            State::Init => match token {
                Token::OpenCurlyBracket => {
                    let _ = val.insert(Value::Object(HashMap::new()));
                    state = State::Object;
                }
                Token::Null | Token::String(_) | Token::Boolean(_) => {
                    let _ = val.insert(token.try_into().expect("token should be valid json value"));
                    state = State::End;
                }
                Token::ClosedCurlyBracket => return Err(Error::Unmatched(token)),
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Object => match token {
                Token::ClosedCurlyBracket => {
                    state = State::End;
                }
                Token::String(s) => {
                    state = State::Key(s);
                }
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Key(s) => match token {
                Token::Colon => state = State::Value { key: s },
                invalid => return Err(Error::UnexpectedToken(invalid)),
            },
            State::Value { key } => match token {
                Token::String(_) | Token::Null | Token::Boolean(_) => {
                    if let Some(Value::Object(ref mut map)) = val {
                        map.insert(key, token.try_into().expect("should be valid json value"));
                        state = State::NextObjectKeyOrFinish;
                    } else {
                        unreachable!("Value must be a map at this point")
                    }
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
                    state = State::Key(s);
                }
                _ => return Err(Error::ExpectedKey),
            },
            State::End => return Err(Error::TokenAfterEnd(token)),
        }
    }

    val.ok_or(Error::Empty)
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
        assert_eq!(parse("{}").unwrap(), kv_to_map(&[]));
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
            (kv_to_map(&[
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
            parse(&format!(
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
        assert_eq!(parse(json), Ok(expected));
    }
}
