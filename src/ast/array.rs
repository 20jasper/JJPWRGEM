use crate::{
    ast::{Value, ValueWithContext, parse_tokens, validate_start_of_value},
    error::{Error, ErrorKind, Result},
    tokens::{Token, TokenWithContext},
};
use core::iter::Peekable;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrayState {
    Open,
    ValueOrEnd {
        items: Vec<Value>,
        open_ctx: TokenWithContext,
    },
    Value {
        items: Vec<Value>,
        open_ctx: TokenWithContext,
        expect_ctx: TokenWithContext,
    },
    CommaOrEnd {
        items: Vec<Value>,
        open_ctx: TokenWithContext,
        last_value_range: Range<usize>,
    },
    End(ValueWithContext),
}

impl ArrayState {
    pub fn process(
        self,
        tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
        text: &str,
    ) -> Result<Self> {
        let next_state = match self {
            ArrayState::Open => match tokens.next() {
                Some(
                    open_ctx @ TokenWithContext {
                        token: Token::OpenSquareBracket,
                        ..
                    },
                ) => ArrayState::ValueOrEnd {
                    items: Vec::new(),
                    open_ctx,
                },
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedOpenBrace {
                            expected: '['.into(),
                            context: None,
                            found: tok,
                        },
                        maybe_token,
                        text,
                    ));
                }
            },

            ArrayState::ValueOrEnd { items, open_ctx } => match tokens.peek().cloned() {
                Some(TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: closed_range,
                    ..
                }) => {
                    tokens.next();
                    ArrayState::End(ValueWithContext::new(
                        Value::Array(items),
                        open_ctx.range.start..closed_range.end,
                    ))
                }
                Some(token_ctx) if token_ctx.token.is_start_of_value() => ArrayState::Value {
                    items,
                    open_ctx: open_ctx.clone(),
                    expect_ctx: open_ctx.clone(),
                },
                Some(_) => {
                    let maybe_token = tokens.next();
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx.clone(), tok)
                                .expect("array should open with a square bracket")
                        },
                        maybe_token,
                        text,
                    ));
                }
                None => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx.clone(), tok)
                                .expect("array should open with a square bracket")
                        },
                        None,
                        text,
                    ));
                }
            },

            ArrayState::Value {
                mut items,
                open_ctx,
                expect_ctx,
            } => {
                validate_start_of_value(text, expect_ctx, tokens.peek().cloned())?;

                let ValueWithContext { value, ctx } = parse_tokens(tokens, text, false)?;
                items.push(value);
                ArrayState::CommaOrEnd {
                    items,
                    open_ctx,
                    last_value_range: ctx,
                }
            }

            ArrayState::CommaOrEnd {
                items, open_ctx, ..
            } => match tokens.peek().cloned() {
                Some(TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: closed_range,
                }) => {
                    tokens.next();
                    ArrayState::End(ValueWithContext::new(
                        Value::Array(items),
                        open_ctx.range.start..closed_range.end,
                    ))
                }
                Some(
                    comma_ctx @ TokenWithContext {
                        token: Token::Comma,
                        ..
                    },
                ) => {
                    tokens.next();
                    ArrayState::Value {
                        items,
                        open_ctx,
                        expect_ctx: comma_ctx,
                    }
                }
                _ => {
                    let maybe_token = tokens.next();
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx.clone(), tok)
                                .expect("array should open with a square bracket")
                        },
                        maybe_token,
                        text,
                    ));
                }
            },

            ArrayState::End(_) => {
                return Ok(self);
            }
        };

        Ok(next_state)
    }
}

pub fn parse_array(
    tokens: &mut Peekable<impl Iterator<Item = TokenWithContext>>,
    text: &str,
) -> Result<ValueWithContext> {
    let mut state = ArrayState::Open;

    loop {
        state = state.process(tokens, text)?;
        if let ArrayState::End(result) = state {
            break Ok(result);
        }
    }
}
