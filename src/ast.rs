use crate::error::{ErrorKind, Result};
use crate::tokens::{Token, TokenWithContext, str_to_tokens};
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
    type Error = crate::ErrorKind;

    fn try_from(token: Token) -> std::result::Result<Self, Self::Error> {
        Ok(match token {
            Token::String(s) => Value::String(s),
            Token::Null => Value::Null,
            Token::Boolean(b) => Value::Boolean(b),
            _ => return Err(ErrorKind::Custom("token is not a valid value".to_owned())),
        })
    }
}

pub fn parse_str(json: &str) -> Result<Value> {
    let tokens = str_to_tokens(json)?;
    parse_tokens(
        &mut tokens
            .into_iter()
            .map(|TokenWithContext { token, .. }| token)
            .peekable(),
        true,
    )
}

pub fn parse_tokens(
    tokens: &mut Peekable<impl Iterator<Item = Token>>,
    fail_on_multiple_value: bool,
) -> Result<Value> {
    let peeked = if let Some(peeked) = tokens.peek() {
        peeked
    } else {
        return Err(ErrorKind::Empty.into());
    };
    let val = match peeked {
        Token::OpenCurlyBrace => parse_object(tokens, fail_on_multiple_value)?,
        Token::Null | Token::String(_) | Token::Boolean(_) => {
            let token = tokens.next().unwrap();
            token.try_into().expect("token should be valid json value")
        }
        invalid => return Err(ErrorKind::ExpectedValue(Some(invalid.clone())).into()),
    };

    if fail_on_multiple_value && let Some(token) = tokens.peek() {
        return Err(ErrorKind::TokenAfterEnd(token.clone()).into());
    }
    Ok(val)
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

impl ObjectState {
    fn process(
        self,
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        fail_on_multiple_value: bool,
    ) -> Result<Self> {
        let res = match self {
            ObjectState::Open => match tokens.next() {
                Some(Token::OpenCurlyBrace) => ObjectState::KeyOrEnd(HashMap::new()),
                invalid => return Err(ErrorKind::ExpectedOpenCurlyBrace(invalid).into()),
            },
            ObjectState::KeyOrEnd(map) => match tokens.next() {
                Some(Token::ClosedCurlyBrace) => ObjectState::End(map),
                Some(Token::String(key)) => ObjectState::Colon { key, map },
                invalid => return Err(ErrorKind::ExpectedKeyOrClosedCurlyBrace(invalid).into()),
            },
            ObjectState::Colon { map, key } => match tokens.next() {
                Some(Token::Colon) => ObjectState::Value { map, key },
                invalid => return Err(ErrorKind::ExpectedColon(invalid).into()),
            },
            ObjectState::Value { mut map, key } => {
                let json_value = match tokens.peek() {
                    Some(
                        Token::OpenCurlyBrace | Token::String(_) | Token::Null | Token::Boolean(_),
                    ) => parse_tokens(tokens, false)?,
                    invalid => return Err(ErrorKind::ExpectedValue(invalid.cloned()).into()),
                };

                map.insert(key, json_value);
                ObjectState::NextKeyOrEnd(map)
            }
            ObjectState::NextKeyOrEnd(map) => match tokens.next() {
                Some(Token::ClosedCurlyBrace) => ObjectState::End(map),
                Some(Token::Comma) => ObjectState::Key(map),
                invalid => {
                    return Err(ErrorKind::ExpectedCommaOrClosedCurlyBrace(invalid.clone()).into());
                }
            },
            ObjectState::Key(map) => match tokens.next() {
                Some(Token::String(key)) => ObjectState::Colon { key, map },
                invalid => return Err(ErrorKind::ExpectedKey(invalid).into()),
            },
            ObjectState::End(map) => {
                if fail_on_multiple_value && let Some(peeked) = tokens.peek() {
                    return Err(ErrorKind::TokenAfterEnd(peeked.clone()).into());
                }
                ObjectState::End(map)
            }
        };

        Ok(res)
    }
}

fn parse_object(
    tokens: &mut Peekable<impl Iterator<Item = Token>>,
    fail_on_multiple_value: bool,
) -> Result<Value> {
    let mut state = ObjectState::Open;

    while tokens.peek().is_some() {
        state = state.process(tokens, fail_on_multiple_value)?;

        if !fail_on_multiple_value && let ObjectState::End(map) = state {
            return Ok(Value::Object(map));
        }
    }

    match state.process(tokens, fail_on_multiple_value) {
        Ok(ObjectState::End(map)) => Ok(Value::Object(map)),
        Err(e) => Err(e),
        _ => unreachable!("object state will always be end or error"),
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
        assert_eq!(parse_str("").unwrap_err(), ErrorKind::Empty.into());
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
            ErrorKind::TokenAfterEnd(Token::OpenCurlyBrace).into()
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
    #[case(r#"{"hi", "#, ErrorKind::ExpectedColon(Some(Token::Comma)))]
    #[case(r#"{"hi""#, ErrorKind::ExpectedColon(None))]
    #[case(r#"{"hi": , "#, ErrorKind::ExpectedValue(Some(Token::Comma)))]
    #[case(r#"{"hi":"#, ErrorKind::ExpectedValue(None))]
    #[case(r#"}"#, ErrorKind::ExpectedValue(Some(Token::ClosedCurlyBrace)))]
    #[case(r#""#, ErrorKind::Empty)]
    #[case(
        r#"{{"#,
        ErrorKind::ExpectedKeyOrClosedCurlyBrace(Some(Token::OpenCurlyBrace))
    )]
    #[case(r#"{"#, ErrorKind::ExpectedKeyOrClosedCurlyBrace(None))]
    #[case(
        r#"{"hi": null null"#,
        ErrorKind::ExpectedCommaOrClosedCurlyBrace(Some(Token::Null))
    )]
    #[case(
        r#"{"hi": null     "#,
        ErrorKind::ExpectedCommaOrClosedCurlyBrace(None)
    )]
    #[case(
        r#"{"hi": null, }"#,
        ErrorKind::ExpectedKey(Some(Token::ClosedCurlyBrace))
    )]
    #[case(r#"{"hi": null, "#, ErrorKind::ExpectedKey(None))]
    fn expected_error(#[case] json: &str, #[case] expected: ErrorKind) {
        assert_eq!(parse_str(json), Err(expected.into()));
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
