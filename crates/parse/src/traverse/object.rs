use core::range::Range;

use crate::{
    Error, ErrorKind, Result,
    tokens::{Token, TokenStream, TokenWithContext},
    traverse::{Visitor, parse_tokens_at_depth, validate_start_of_value},
};

#[derive(Debug, PartialEq, Eq, Clone)]
enum ObjectState {
    Open,
    KeyOrEnd {
        open_ctx: TokenWithContext,
        last_pair: Option<Range<usize>>,
    },
    Key {
        comma_ctx: TokenWithContext,
        open_ctx: TokenWithContext,
    },
    Colon {
        key_ctx: TokenWithContext,
        open_ctx: TokenWithContext,
    },
    Value {
        colon_ctx: TokenWithContext,
        open_ctx: TokenWithContext,
    },
    End(Range<usize>),
}

impl ObjectState {
    fn process<'a>(
        self,
        tokens: &mut TokenStream<'a>,
        text: &'a str,
        visitor: &mut impl Visitor<'a>,
        depth: usize,
    ) -> Result<Self> {
        let res = match self {
            ObjectState::Open => match tokens.next_token()? {
                Some(
                    ctx @ TokenWithContext {
                        token: Token::OpenCurlyBrace,
                        ..
                    },
                ) => {
                    visitor.on_object_open(ctx.range);
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
                    Some(
                        ctx @ TokenWithContext {
                            token: Token::ClosedCurlyBrace,
                            ..
                        },
                    ),
                ) => {
                    visitor.on_object_close(ctx.range);
                    ObjectState::End(open_ctx.range.start..ctx.range.end)
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
                            token: Token::String,
                            ..
                        },
                    ),
                ) => {
                    let body = key_ctx.content_range();
                    visitor.on_object_key(body, &text[body]);
                    ObjectState::Colon { key_ctx, open_ctx }
                }
                (Some(pair_span), maybe_token) => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                            range: pair_span,
                            open_ctx,
                            found: tok,
                        },
                        maybe_token,
                        text,
                    ));
                }
                (None, maybe_token) => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx, tok)
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
                        token: Token::String,
                        ..
                    },
                ) => {
                    let body = key_ctx.content_range();
                    visitor.on_object_key(body, &text[body]);
                    ObjectState::Colon { key_ctx, open_ctx }
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedKey(comma_ctx, tok),
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
                        |tok| ErrorKind::ExpectedColon(key_ctx, tok),
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::Value {
                colon_ctx,
                open_ctx,
            } => {
                validate_start_of_value(text, colon_ctx, tokens.peek_token()?.copied())?;

                let value_range = parse_tokens_at_depth(tokens, text, false, visitor, depth + 1)?;

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
    depth: usize,
    visitor: &mut impl Visitor<'a>,
) -> Result<Range<usize>> {
    let mut state = ObjectState::Open;

    loop {
        state = state.process(tokens, text, visitor, depth)?;
        if let ObjectState::End(range) = state {
            break Ok(range);
        }
    }
}
