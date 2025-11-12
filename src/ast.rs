mod object;

use crate::Error;
use crate::ast::object::parse_object;
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
    parse_tokens(&mut tokens.into_iter().peekable(), json, true)
}

pub fn parse_tokens(
    tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    text: &str,
    fail_on_multiple_value: bool,
) -> Result<Value> {
    let peeked = if let Some(peeked) = tokens.peek() {
        peeked
    } else {
        return Err(Error::from_maybe_token_with_context(
            ErrorKind::ExpectedValue,
            None,
            text,
        ));
    };
    let val = match &peeked.token {
        Token::OpenCurlyBrace => parse_object(tokens, text, fail_on_multiple_value)?,
        Token::Null | Token::String(_) | Token::Boolean(_) => {
            let TokenWithContext { token, .. } = tokens.next().unwrap();
            token.try_into().expect("token should be valid json value")
        }
        invalid => {
            return Err(Error::new(
                ErrorKind::ExpectedValue(Some(invalid.clone())),
                peeked.range.clone(),
                text,
            ));
        }
    };

    if fail_on_multiple_value && let Some(TokenWithContext { token, range }) = tokens.peek() {
        return Err(Error::new(
            ErrorKind::TokenAfterEnd(token.clone()),
            range.clone(),
            text,
        ));
    }
    Ok(val)
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

    fn json_to_json_and_error(
        json: &'static str,
        kind: ErrorKind,
        range: std::ops::Range<usize>,
    ) -> (&'static str, Error) {
        (json, Error::new(kind, range, json))
    }

    #[rstest::rstest]
    #[case(json_to_json_and_error(
        r#"{"hi", "#,
        ErrorKind::ExpectedColon(Some(Token::Comma)),
        5..6,
    ))]
    #[case(json_to_json_and_error(
        r#"  {"hi"    "#,
        ErrorKind::ExpectedColon(None),
        6..7,
    ))]
    #[case(json_to_json_and_error(
        r#"{"hi"    "#,
        ErrorKind::ExpectedColon(None),
        4..5,
    ))]
    #[case(json_to_json_and_error(
        r#"{"hi":"#,
        ErrorKind::ExpectedValue(None),
        5..6,
    ))]
    #[case(json_to_json_and_error(
        r#"}"#,
        ErrorKind::ExpectedValue(Some(Token::ClosedCurlyBrace)),
        0..1,
    ))]
    #[case(json_to_json_and_error(
        r#""#,
        ErrorKind::ExpectedValue(None),
        0..0,
    ))]
    #[case(json_to_json_and_error(
        r#"{{"#,
        ErrorKind::ExpectedKeyOrClosedCurlyBrace(Some(Token::OpenCurlyBrace)),
        1..2,
    ))]
    #[case(json_to_json_and_error(
        r#"{"#,
        ErrorKind::ExpectedKeyOrClosedCurlyBrace(None),
        0..1,
    ))]
    #[case(json_to_json_and_error(
        r#"{"hi": null null"#,
        ErrorKind::ExpectedCommaOrClosedCurlyBrace(Some(Token::Null)),
        12..16,
    ))]
    #[case(json_to_json_and_error(
        r#"{"hi": null     "#,
        ErrorKind::ExpectedCommaOrClosedCurlyBrace(None),
        10..11,
    ))]
    #[case(json_to_json_and_error(
        r#"{"hi": null, }"#,
        ErrorKind::ExpectedKey(Some(Token::ClosedCurlyBrace)),
        13..14,
    ))]
    #[case(json_to_json_and_error(
        r#"{"hi": null, "#,
        ErrorKind::ExpectedKey(None),
        11..12,
    ))]
    #[case(json_to_json_and_error(
        r#"{}{"#,
        ErrorKind::TokenAfterEnd(Token::OpenCurlyBrace),
        2..3,
    ))]
    fn expected_error(#[case] (json, expected): (&str, Error)) {
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
