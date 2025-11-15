use core::iter::Peekable;
use std::collections::HashMap;

use crate::{
    Error, ErrorKind, Result,
    ast::{Value, parse_tokens},
    tokens::{Token, TokenOption, TokenWithContext},
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum ObjectState {
    Open,
    KeyOrEnd(HashMap<String, Value>, TokenWithContext),
    NextKeyOrEnd(HashMap<String, Value>),
    Key(HashMap<String, Value>, TokenWithContext),
    Colon {
        key_ctx: TokenWithContext,
        key: String,
        map: HashMap<String, Value>,
    },
    Value {
        colon_ctx: TokenWithContext,
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
                Some(
                    ctx @ TokenWithContext {
                        token: Token::OpenCurlyBrace,
                        ..
                    },
                ) => ObjectState::KeyOrEnd(HashMap::new(), ctx),
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedOpenCurlyBrace,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::KeyOrEnd(map, ctx) => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    ..
                }) => ObjectState::End(map),
                Some(TokenWithContext {
                    token: Token::String(key),
                    range: key_range,
                }) => {
                    let key_ctx = TokenWithContext {
                        token: Token::String(key.clone()),
                        range: key_range,
                    };
                    ObjectState::Colon { key_ctx, key, map }
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok: TokenOption| {
                            ErrorKind::ExpectedKeyOrClosedCurlyBrace(ctx.clone(), tok)
                        },
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Colon { map, key, key_ctx } => match tokens.next() {
                Some(
                    colon_ctx @ TokenWithContext {
                        token: Token::Colon,
                        ..
                    },
                ) => ObjectState::Value {
                    map,
                    key,
                    colon_ctx,
                },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedColon(key_ctx.clone(), tok),
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Value {
                mut map,
                key,
                colon_ctx,
            } => {
                let json_value = match tokens.peek() {
                    Some(TokenWithContext {
                        token:
                            Token::OpenCurlyBrace | Token::String(_) | Token::Null | Token::Boolean(_),
                        ..
                    }) => parse_tokens(tokens, text, false)?,
                    maybe_token => {
                        return Err(Error::from_maybe_token_with_context(
                            |tok| ErrorKind::ExpectedValue(Some(colon_ctx.clone()), tok),
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
                Some(
                    ctx @ TokenWithContext {
                        token: Token::Comma,
                        ..
                    },
                ) => ObjectState::Key(map, ctx),
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        ErrorKind::ExpectedCommaOrClosedCurlyBrace,
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Key(map, ctx) => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::String(key),
                    range: key_range,
                }) => {
                    let key_ctx = TokenWithContext {
                        token: Token::String(key.clone()),
                        range: key_range,
                    };
                    ObjectState::Colon { key_ctx, key, map }
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok: TokenOption| ErrorKind::ExpectedKey(ctx.clone(), tok),
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
