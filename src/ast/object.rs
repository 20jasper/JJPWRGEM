use crate::{
    Error, ErrorKind, Result,
    ast::{Value, ValueWithContext, parse_tokens},
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
        last_pair: Option<Range<usize>>,
    },
    Key {
        map: HashMap<String, Value>,
        comma_ctx: TokenWithContext,
        open_ctx: TokenWithContext,
    },
    Colon {
        key_ctx: TokenWithContext,
        map: HashMap<String, Value>,
        open_ctx: TokenWithContext,
    },
    Value {
        key_ctx: TokenWithContext,
        colon_ctx: TokenWithContext,
        map: HashMap<String, Value>,
        open_ctx: TokenWithContext,
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
                    last_pair: None,
                },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedOpenCurlyBrace(None, tok),
                        maybe_token.clone(),
                        text,
                    ));
                }
            },

            ObjectState::KeyOrEnd {
                map,
                open_ctx,
                last_pair,
            } => match (last_pair, tokens.next()) {
                (
                    _,
                    Some(TokenWithContext {
                        token: Token::ClosedCurlyBrace,
                        ..
                    }),
                ) => ObjectState::End(map),
                (
                    Some(_),
                    Some(
                        comma_ctx @ TokenWithContext {
                            token: Token::Comma,
                            ..
                        },
                    ),
                ) => ObjectState::Key {
                    map,
                    comma_ctx,
                    open_ctx,
                },
                (
                    None,
                    Some(
                        key_ctx @ TokenWithContext {
                            token: Token::String(_),
                            ..
                        },
                    ),
                ) => ObjectState::Colon {
                    key_ctx,
                    map,
                    open_ctx,
                },
                (Some(pair_span), maybe_token) => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                            range: pair_span.clone(),
                            open_ctx: open_ctx.clone(),
                            found: tok,
                        },
                        maybe_token,
                        text,
                    ));
                }
                (None, maybe_token) => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok: TokenOption| {
                            ErrorKind::ExpectedKeyOrClosedCurlyBrace(open_ctx.clone(), tok)
                        },
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::Key {
                map,
                comma_ctx,
                open_ctx,
            } => match tokens.next() {
                Some(
                    key_ctx @ TokenWithContext {
                        token: Token::String(_),
                        ..
                    },
                ) => ObjectState::Colon {
                    key_ctx,
                    map,
                    open_ctx,
                },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok: TokenOption| ErrorKind::ExpectedKey(comma_ctx.clone(), tok),
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Colon {
                map,
                key_ctx,
                open_ctx,
            } => match tokens.next() {
                Some(
                    colon_ctx @ TokenWithContext {
                        token: Token::Colon,
                        ..
                    },
                ) => ObjectState::Value {
                    map,
                    key_ctx,
                    colon_ctx,
                    open_ctx,
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
                open_ctx,
            } => {
                let peeked = tokens.peek().cloned();
                if !matches!(
                    peeked.as_ref().map(|ctx| &ctx.token),
                    Some(
                        Token::OpenCurlyBrace | Token::String(_) | Token::Null | Token::Boolean(_)
                    )
                ) {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedValue(Some(colon_ctx.clone()), tok),
                        peeked,
                        text,
                    ));
                }

                let Token::String(key) = key_ctx.token else {
                    unreachable!("key context should always be a string");
                };

                let ValueWithContext {
                    value: json_value,
                    ctx: json_ctx,
                } = parse_tokens(tokens, text, false)?;
                map.insert(key, json_value);

                ObjectState::KeyOrEnd {
                    map,
                    open_ctx,
                    last_pair: Some(colon_ctx.range.start..json_ctx.end),
                }
            }
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
