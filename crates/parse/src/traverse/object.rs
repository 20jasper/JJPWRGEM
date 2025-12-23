use crate::{
    Error, ErrorKind, Result,
    tokens::{Token, TokenOption, TokenStream, TokenWithContext},
    traverse::{ParseVisitor, parse_tokens, validate_start_of_value},
};
use core::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
enum ObjectState<'a> {
    Open,
    KeyOrEnd {
        open_ctx: TokenWithContext<'a>,
        last_pair: Option<Range<usize>>,
    },
    Key {
        comma_ctx: TokenWithContext<'a>,
        open_ctx: TokenWithContext<'a>,
    },
    Colon {
        key_ctx: TokenWithContext<'a>,
        open_ctx: TokenWithContext<'a>,
    },
    Value {
        colon_ctx: TokenWithContext<'a>,
        open_ctx: TokenWithContext<'a>,
    },
    End(Range<usize>),
}

impl<'a> ObjectState<'a> {
    fn process(
        self,
        tokens: &mut TokenStream<'a>,
        text: &'a str,
        visitor: &mut impl ParseVisitor<'a>,
    ) -> Result<'a, Self> {
        let res = match self {
            ObjectState::Open => match tokens.next_token()? {
                Some(
                    ctx @ TokenWithContext {
                        token: Token::OpenCurlyBrace,
                        ..
                    },
                ) => {
                    visitor.on_object_open(ctx.clone());
                    ObjectState::KeyOrEnd {
                        open_ctx: ctx,
                        last_pair: None,
                    }
                }
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
                open_ctx,
                last_pair,
            } => match (last_pair, tokens.next_token()?) {
                (
                    _,
                    Some(TokenWithContext {
                        token: Token::ClosedCurlyBrace,
                        range: closed_range,
                    }),
                ) => {
                    let range = open_ctx.range.start..closed_range.end;
                    visitor.on_object_close(range.clone());
                    ObjectState::End(range)
                }
                (
                    Some(_),
                    Some(
                        comma_ctx @ TokenWithContext {
                            token: Token::Comma,
                            ..
                        },
                    ),
                ) => {
                    visitor.on_item_delim();
                    ObjectState::Key {
                        comma_ctx,
                        open_ctx,
                    }
                }
                (
                    None,
                    Some(
                        key_ctx @ TokenWithContext {
                            token: Token::String(key),
                            ..
                        },
                    ),
                ) => {
                    visitor.on_object_key(key);
                    ObjectState::Colon { key_ctx, open_ctx }
                }
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
                comma_ctx,
                open_ctx,
            } => match tokens.next_token()? {
                Some(
                    key_ctx @ TokenWithContext {
                        token: Token::String(key),
                        ..
                    },
                ) => {
                    visitor.on_object_key(key);
                    ObjectState::Colon { key_ctx, open_ctx }
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok: TokenOption| ErrorKind::ExpectedKey(comma_ctx.clone(), tok),
                        maybe_token,
                        text,
                    ));
                }
            },
            ObjectState::Colon { key_ctx, open_ctx } => match tokens.next_token()? {
                Some(
                    colon_ctx @ TokenWithContext {
                        token: Token::Colon,
                        ..
                    },
                ) => {
                    visitor.on_object_key_val_delim();
                    ObjectState::Value {
                        colon_ctx,
                        open_ctx,
                    }
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedColon(key_ctx.clone(), tok),
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::Value {
                colon_ctx,
                open_ctx,
            } => {
                validate_start_of_value(text, colon_ctx.clone(), tokens.peek_token()?.cloned())?;

                let value_range = parse_tokens(tokens, text, false, visitor)?;

                ObjectState::KeyOrEnd {
                    open_ctx,
                    last_pair: Some(colon_ctx.range.start..value_range.end),
                }
            }
            ObjectState::End(range) => ObjectState::End(range),
        };

        Ok(res)
    }
}

pub fn parse_object<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    visitor: &mut impl ParseVisitor<'a>,
) -> Result<'a, Range<usize>> {
    let mut state = ObjectState::Open;

    loop {
        state = state.process(tokens, text, visitor)?;
        if let ObjectState::End(range) = state {
            break Ok(range);
        }
    }
}
