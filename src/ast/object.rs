use crate::{
    Error, ErrorKind, Result,
    ast::{Value, parse_tokens},
    tokens::{Token, TokenOption, TokenWithContext},
};
use core::iter::Peekable;
use core::ops::Range;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
enum ObjectState {
    Open,
    KeyOrEnd {
        map: HashMap<String, Value>,
        open_ctx: TokenWithContext,
    },
    NextKeyOrEnd {
        map: HashMap<String, Value>,
        colon_range: Range<usize>,
        value_end: usize,
    },
    Key(HashMap<String, Value>, TokenWithContext),
    Colon {
        key_ctx: TokenWithContext,
        map: HashMap<String, Value>,
    },
    Value {
        key_ctx: TokenWithContext,
        colon_ctx: TokenWithContext,
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
                ) => ObjectState::KeyOrEnd {
                    map: HashMap::new(),
                    open_ctx: ctx,
                },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedOpenCurlyBrace(None, tok),
                        maybe_token.clone(),
                        text,
                    ));
                }
            },
            ObjectState::KeyOrEnd { map, open_ctx } => match tokens.next() {
                Some(TokenWithContext {
                    token: Token::ClosedCurlyBrace,
                    ..
                }) => ObjectState::End(map),
                Some(
                    key_ctx @ TokenWithContext {
                        token: Token::String(_),
                        ..
                    },
                ) => ObjectState::Colon { key_ctx, map },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok: TokenOption| {
                            ErrorKind::ExpectedKeyOrClosedCurlyBrace(open_ctx.clone(), tok)
                        },
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Colon { map, key_ctx } => match tokens.next() {
                Some(
                    colon_ctx @ TokenWithContext {
                        token: Token::Colon,
                        ..
                    },
                ) => ObjectState::Value {
                    map,
                    key_ctx,
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
                key_ctx,
                colon_ctx,
            } => {
                let value_ctx = match tokens.peek() {
                    Some(
                        ctx @ TokenWithContext {
                            token:
                                Token::OpenCurlyBrace
                                | Token::String(_)
                                | Token::Null
                                | Token::Boolean(_),
                            ..
                        },
                    ) => ctx.clone(),
                    maybe_token => {
                        return Err(Error::from_maybe_token_with_context(
                            |tok| ErrorKind::ExpectedValue(Some(colon_ctx.clone()), tok),
                            maybe_token.cloned(),
                            text,
                        ));
                    }
                };

                let json_value = parse_tokens(tokens, text, false)?;

                if let Token::String(key) = &key_ctx.token {
                    map.insert(key.clone(), json_value);
                } else {
                    unreachable!("token already matched as string");
                }
                let value_end = value_ctx.range.end;

                ObjectState::NextKeyOrEnd {
                    map,
                    colon_range: colon_ctx.range.clone(),
                    value_end,
                }
            }
            ObjectState::NextKeyOrEnd {
                map,
                colon_range,
                value_end,
            } => match tokens.next() {
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
                        |tok| {
                            ErrorKind::ExpectedCommaOrClosedCurlyBrace(
                                colon_range.start..value_end,
                                tok,
                            )
                        },
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Key(map, ctx) => match tokens.next() {
                Some(
                    key_ctx @ TokenWithContext {
                        token: Token::String(_),
                        ..
                    },
                ) => ObjectState::Colon { key_ctx, map },
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
