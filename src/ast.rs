mod array;
mod object;

use crate::Error;
use crate::ast::array::parse_array;
use crate::ast::object::parse_object;
use crate::error::{ErrorKind, Result};
use crate::tokens::{Token, TokenWithContext, str_to_tokens};
use core::iter::Peekable;
use core::ops::Range;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Null,
    String(String),
    Number(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Boolean(bool),
}

fn token_to_value(token: Token) -> Option<Value> {
    Some(match token {
        Token::String(s) => Value::String(s),
        Token::Null => Value::Null,
        Token::Boolean(b) => Value::Boolean(b),
        Token::Number(n) => Value::Number(n),
        _ => return None,
    })
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValueWithContext {
    value: Value,
    range: Range<usize>,
}

impl ValueWithContext {
    pub fn new(value: Value, range: Range<usize>) -> Self {
        Self { value, range }
    }
}

pub fn parse_str(json: &str) -> Result<Value> {
    let tokens = str_to_tokens(json)?;
    Ok(parse_tokens(&mut tokens.into_iter().peekable(), json, true)?.value)
}

pub fn parse_tokens(
    tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    text: &str,
    fail_on_multiple_value: bool,
) -> Result<ValueWithContext> {
    let peeked = if let Some(peeked) = tokens.peek() {
        peeked.clone()
    } else {
        return Err(Error::from_maybe_token_with_context(
            |tok| ErrorKind::ExpectedValue(None, tok),
            None,
            text,
        ));
    };
    let val = match &peeked.token {
        Token::OpenCurlyBrace => parse_object(tokens, text)?,
        Token::OpenSquareBracket => parse_array(tokens, text)?,
        Token::Null | Token::String(_) | Token::Boolean(_) | Token::Number(_) => {
            let TokenWithContext { token, range } = tokens.next().unwrap();
            ValueWithContext {
                value: token_to_value(token).expect("token should be valid json value"),
                range,
            }
        }
        invalid => {
            return Err(Error::new(
                ErrorKind::ExpectedValue(None, Some(invalid.clone()).into()),
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

fn validate_start_of_value(
    text: &str,
    expect_ctx: TokenWithContext,
    maybe_token: Option<TokenWithContext>,
) -> Result<()> {
    if !maybe_token
        .as_ref()
        .is_some_and(|ctx| ctx.token.is_start_of_value())
    {
        Err(Error::from_maybe_token_with_context(
            |tok| ErrorKind::ExpectedValue(Some(expect_ctx.clone()), tok),
            maybe_token,
            text,
        ))
    } else {
        Ok(())
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
