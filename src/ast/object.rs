use core::iter::Peekable;
use std::collections::HashMap;

use crate::{
    Error, ErrorKind, Result,
    ast::{Value, parse_tokens},
    tokens::{Token, TokenWithContext},
};

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
        tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
        text: &str,
        fail_on_multiple_value: bool,
    ) -> Result<Self> {
        let res = match self {
            ObjectState::Open => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::OpenCurlyBrace,
                    ..
                }) => ObjectState::KeyOrEnd(HashMap::new()),
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedOpenCurlyBrace,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::KeyOrEnd(map) => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    ..
                }) => ObjectState::End(map),
                Some(TokenWithContext {
                    token: Token::String(key),
                    ..
                }) => ObjectState::Colon { key, map },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedKeyOrClosedCurlyBrace,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Colon { map, key } => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::Colon,
                    ..
                }) => ObjectState::Value { map, key },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedColon,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Value { mut map, key } => {
                let json_value = match tokens.peek() {
                    Some(TokenWithContext {
                        token:
                            Token::OpenCurlyBrace | Token::String(_) | Token::Null | Token::Boolean(_),
                        ..
                    }) => parse_tokens(tokens, text, false)?,
                    maybe_token => {
                        return Err(Error::from_maybe_token_with_context(
                            ErrorKind::ExpectedValue,
                            maybe_token.cloned(),
                            text,
                        ));
                    }
                };

                map.insert(key, json_value);
                ObjectState::NextKeyOrEnd(map)
            }
            ObjectState::NextKeyOrEnd(map) => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    ..
                }) => ObjectState::End(map),
                Some(TokenWithContext {
                    token: Token::Comma,
                    ..
                }) => ObjectState::Key(map),
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedCommaOrClosedCurlyBrace,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Key(map) => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::String(key),
                    ..
                }) => ObjectState::Colon { key, map },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedKey,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::End(map) => {
                if fail_on_multiple_value
                    && let Some(TokenWithContext { token, range }) = tokens.peek().cloned()
                {
                    return Err(Error::new(ErrorKind::TokenAfterEnd(token), range, text));
                }
                ObjectState::End(map)
            }
        };

        Ok(res)
    }
}

pub fn parse_object(
    tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    text: &str,
    fail_on_multiple_value: bool,
) -> Result<Value> {
    let mut state = ObjectState::Open;

    while tokens.peek().is_some() {
        state = state.process(tokens, text, fail_on_multiple_value)?;

        if !fail_on_multiple_value && let ObjectState::End(map) = state {
            return Ok(Value::Object(map));
        }
    }

    match state.process(tokens, text, fail_on_multiple_value) {
        Ok(ObjectState::End(map)) => Ok(Value::Object(map)),
        Err(e) => Err(e),
        _ => unreachable!("object state will always be end or error"),
    }
}
