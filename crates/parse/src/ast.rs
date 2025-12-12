mod array;
mod object;

use crate::ast::{array::parse_array, object::parse_object};
use crate::tokens::{Token, TokenStream, TokenWithContext};
use crate::{Error, ErrorKind, Result};
use core::ops::Range;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ObjectEntries<'a>(pub Vec<(&'a str, Value<'a>)>);

impl<'a> ObjectEntries<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, k: &'a str, v: Value<'a>) {
        self.0.push((k, v));
    }

    pub fn get(&self, k: &'a str) -> Option<&Value<'a>> {
        self.0.iter().find_map(|(k2, v)| (k == *k2).then_some(v))
    }
}

impl<'a> From<Vec<(&'a str, Value<'a>)>> for ObjectEntries<'a> {
    fn from(value: Vec<(&'a str, Value<'a>)>) -> Self {
        ObjectEntries(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value<'a> {
    Null,
    String(&'a str),
    Number(Cow<'a, str>),
    Object(ObjectEntries<'a>),
    Array(Vec<Value<'a>>),
    Boolean(bool),
}

fn token_to_value<'a>(token: Token<'a>) -> Option<Value<'a>> {
    Some(match token {
        Token::String(s) => Value::String(s),
        Token::Null => Value::Null,
        Token::Boolean(b) => Value::Boolean(b),
        Token::Number(n) => Value::Number(n),
        _ => return None,
    })
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValueWithContext<'a> {
    value: Value<'a>,
    range: Range<usize>,
}

impl<'a> ValueWithContext<'a> {
    pub fn new(value: Value<'a>, range: Range<usize>) -> Self {
        Self { value, range }
    }
}

pub fn parse_str<'a>(json: &'a str) -> Result<'a, Value<'a>> {
    Ok(parse_tokens(&mut TokenStream::new(json), json, true)?.value)
}

pub fn parse_tokens<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    fail_on_multiple_value: bool,
) -> Result<'a, ValueWithContext<'a>> {
    let Some(peeked) = tokens.peek_token()? else {
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
            let TokenWithContext { token, range } = tokens
                .next_token()?
                .expect("peek guaranteed a value for scalar token");
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

    if fail_on_multiple_value
        && let Some(TokenWithContext { token, range }) = tokens.peek_token()?
    {
        return Err(Error::new(
            ErrorKind::TokenAfterEnd(token.clone()),
            range.clone(),
            text,
        ));
    }

    Ok(val)
}

fn validate_start_of_value<'a>(
    text: &'a str,
    expect_ctx: TokenWithContext<'a>,
    maybe_token: Option<TokenWithContext<'a>>,
) -> Result<'a, ()> {
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

    fn kv_to_map<'a>(tuples: &[(&'a str, Value<'a>)]) -> Value<'a> {
        Value::Object(tuples.to_vec().into())
    }

    #[test]
    fn empty_object() {
        assert_eq!(parse_str("{}").unwrap(), kv_to_map(&[]));
    }

    #[test]
    fn one_key_value_pair() {
        assert_eq!(
            parse_str(r#"{"hi":"bye"}"#).unwrap(),
            kv_to_map(&[("hi", Value::String("bye"))])
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
            nested(nested(nested(nested(Value::String("rust")))))
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
