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
    End(HashMap<String, Value>, Range<usize>),
}

impl ObjectState {
    fn process(
        self,
        tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
        text: &str,
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
                        range: closed_range,
                    }),
                ) => ObjectState::End(map, open_ctx.range.start..closed_range.end),
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
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx.clone(), tok)
                                .expect("object should open with a curly brace")
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
                if !peeked
                    .as_ref()
                    .is_some_and(|ctx| ctx.token.is_start_of_value())
                {
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
            ObjectState::End(map, range) => ObjectState::End(map, range),
        };

        Ok(res)
    }
}

pub fn parse_object(
    tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    text: &str,
) -> Result<ValueWithContext> {
    let mut state = ObjectState::Open;

    loop {
        state = state.process(tokens, text)?;
        if let ObjectState::End(map, range) = state {
            break Ok(ValueWithContext::new(Value::Object(map), range));
        }
    }
}
