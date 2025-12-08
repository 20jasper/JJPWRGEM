use crate::{
    Error, ErrorKind, Result,
    ast::{ObjectEntries, Value, ValueWithContext, parse_tokens, validate_start_of_value},
    tokens::{Token, TokenOption, TokenStream, TokenWithContext},
};
use core::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
enum ObjectState<'a> {
    Open,
    KeyOrEnd {
        map: ObjectEntries<'a>,
        open_ctx: TokenWithContext<'a>,
        last_pair: Option<Range<usize>>,
    },
    Key {
        map: ObjectEntries<'a>,
        comma_ctx: TokenWithContext<'a>,
        open_ctx: TokenWithContext<'a>,
    },
    Colon {
        key_ctx: TokenWithContext<'a>,
        map: ObjectEntries<'a>,
        open_ctx: TokenWithContext<'a>,
    },
    Value {
        key_ctx: TokenWithContext<'a>,
        colon_ctx: TokenWithContext<'a>,
        map: ObjectEntries<'a>,
        open_ctx: TokenWithContext<'a>,
    },
    End(ObjectEntries<'a>, Range<usize>),
}

impl<'a> ObjectState<'a> {
    fn process(self, tokens: &mut TokenStream<'a>, text: &'a str) -> Result<'a, Self> {
        let res = match self {
            ObjectState::Open => match tokens.next_token()? {
                Some(
                    ctx @ TokenWithContext {
                        token: Token::OpenCurlyBrace,
                        ..
                    },
                ) => ObjectState::KeyOrEnd {
                    map: ObjectEntries::new(),
                    open_ctx: ctx,
                    last_pair: None,
                },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedOpenBrace {
                            expected: '{'.into(),
                            context: None,
                            found: tok,
                        },
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::KeyOrEnd {
                map,
                open_ctx,
                last_pair,
            } => match (last_pair, tokens.next_token()?) {
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
            } => match tokens.next_token()? {
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
            } => match tokens.next_token()? {
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
                validate_start_of_value(text, colon_ctx.clone(), tokens.peek_token()?)?;

                let Token::String(key) = key_ctx.token else {
                    unreachable!("key context should always be a string");
                };

                let ValueWithContext {
                    value: json_value,
                    range: json_range,
                } = parse_tokens(tokens, text, false)?;
                map.push(key, json_value);

                ObjectState::KeyOrEnd {
                    map,
                    open_ctx,
                    last_pair: Some(colon_ctx.range.start..json_range.end),
                }
            }
            ObjectState::End(map, range) => ObjectState::End(map, range),
        };

        Ok(res)
    }
}

pub fn parse_object<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
) -> Result<'a, ValueWithContext<'a>> {
    let mut state = ObjectState::Open;

    loop {
        state = state.process(tokens, text)?;
        if let ObjectState::End(map, range) = state {
            break Ok(ValueWithContext::new(Value::Object(map), range));
        }
    }
}
