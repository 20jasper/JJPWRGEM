use core::range::Range;

use crate::{
    Error, ErrorKind, Result,
    tokens::{Token, TokenStream, TokenWithContext, byte_range},
    traverse::{Visitor, parse_tokens_at_depth, punctuation_at, validate_start_of_value},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum ObjectState {
    KeyOrEnd,
    CommaOrEnd,
    Key,
    Colon,
    Value,
    End,
}

struct ObjectOffsets {
    open_start: usize,
    delim: usize,
    key: Range<usize>,
    last_pair: Range<usize>,
}

fn open_ctx(open_start: usize) -> TokenWithContext {
    TokenWithContext::new(Token::OpenCurlyBrace, byte_range(open_start))
}

impl ObjectState {
    fn process<'a>(
        self,
        tokens: &mut TokenStream<'a>,
        text: &'a str,
        visitor: &mut impl Visitor<'a>,
        depth: usize,
        offsets: &mut ObjectOffsets,
    ) -> Result<Self> {
        let res = match self {
            ObjectState::KeyOrEnd => match tokens.next_token()? {
                Some(
                    ctx @ TokenWithContext {
                        token: Token::ClosedCurlyBrace,
                        ..
                    },
                ) => {
                    visitor.on_object_close(ctx.range);
                    ObjectState::End
                }
                Some(
                    key_ctx @ TokenWithContext {
                        token: Token::String,
                        ..
                    },
                ) => {
                    let body = key_ctx.content_range();
                    visitor.on_object_key(body, &text[body]);
                    offsets.key = key_ctx.range;
                    ObjectState::Colon
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(
                                open_ctx(offsets.open_start),
                                tok,
                            )
                            .expect("object should open with a curly brace")
                        },
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::CommaOrEnd => match tokens.next_token()? {
                Some(
                    ctx @ TokenWithContext {
                        token: Token::ClosedCurlyBrace,
                        ..
                    },
                ) => {
                    visitor.on_object_close(ctx.range);
                    ObjectState::End
                }
                Some(TokenWithContext {
                    token: Token::Comma,
                    range: comma_range,
                }) => {
                    visitor.on_item_delim();
                    offsets.delim = comma_range.start;
                    ObjectState::Key
                }
                maybe_token => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                            range: offsets.last_pair,
                            open_ctx: open_ctx(offsets.open_start),
                            found: tok,
                        },
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::Key => match tokens.next_token()? {
                Some(
                    key_ctx @ TokenWithContext {
                        token: Token::String,
                        ..
                    },
                ) => {
                    let body = key_ctx.content_range();
                    visitor.on_object_key(body, &text[body]);
                    offsets.key = key_ctx.range;
                    ObjectState::Colon
                }
                maybe_token => {
                    let comma_ctx = punctuation_at(text, offsets.delim);
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedKey(comma_ctx, tok),
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::Colon => match tokens.next_token()? {
                Some(
                    colon_ctx @ TokenWithContext {
                        token: Token::Colon,
                        ..
                    },
                ) => {
                    visitor.on_object_key_val_delim();
                    offsets.last_pair = colon_ctx.range.start..colon_ctx.range.end;
                    ObjectState::Value
                }
                maybe_token => {
                    let key_ctx = TokenWithContext::new(Token::String, offsets.key);
                    return Err(Error::from_maybe_token_with_context(
                        |tok| ErrorKind::ExpectedColon(key_ctx, tok),
                        maybe_token,
                        text,
                    ));
                }
            },

            ObjectState::Value => {
                let peeked = tokens.peek_token()?.copied();
                validate_start_of_value(
                    text,
                    || punctuation_at(text, offsets.last_pair.start),
                    peeked,
                )?;

                let value_range = parse_tokens_at_depth(tokens, text, false, visitor, depth + 1)?;
                offsets.last_pair = offsets.last_pair.start..value_range.end;
                ObjectState::CommaOrEnd
            }

            ObjectState::End => self,
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
    let open_start = match tokens.next_token()? {
        Some(
            ctx @ TokenWithContext {
                token: Token::OpenCurlyBrace,
                ..
            },
        ) => {
            visitor.on_object_open(ctx.range);
            ctx.range.start
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
    };

    let mut offsets = ObjectOffsets {
        open_start,
        delim: open_start,
        key: open_start..open_start,
        last_pair: open_start..open_start,
    };
    let mut state = ObjectState::KeyOrEnd;
    while state != ObjectState::End {
        state = state.process(tokens, text, visitor, depth, &mut offsets)?;
    }

    // the cursor sits right after the closing `}`
    Ok(open_start..tokens.pos())
}
