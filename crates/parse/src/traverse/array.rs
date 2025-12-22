use crate::{
    error::{Error, ErrorKind, Result},
    tokens::{Token, TokenStream, TokenWithContext},
    traverse::{ParseVisitor, parse_tokens, validate_start_of_value},
};
use std::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArrayState<'a> {
    Open,
    ValueOrEnd {
        open_ctx: TokenWithContext<'a>,
    },
    Value {
        open_ctx: TokenWithContext<'a>,
        expect_ctx: TokenWithContext<'a>,
    },
    CommaOrEnd {
        open_ctx: TokenWithContext<'a>,
        last_value_range: Range<usize>,
    },
    End(Range<usize>),
}

impl<'a> ArrayState<'a> {
    pub fn process<H: ParseVisitor<'a>>(
        self,
        tokens: &mut TokenStream<'a>,
        text: &'a str,
        visitor: &mut H,
    ) -> Result<'a, Self> {
        let next_state = match self {
            ArrayState::Open => match tokens.next_token()? {
                Some(
                    open_ctx @ TokenWithContext {
                        token: Token::OpenSquareBracket,
                        ..
                    },
                ) => {
                    visitor.on_array_open(open_ctx.clone());
                    ArrayState::ValueOrEnd { open_ctx }
                }
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

            ArrayState::ValueOrEnd { open_ctx } => match tokens.peek_token()?.cloned() {
                Some(TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: closed_range,
                    ..
                }) => {
                    tokens.next_token()?;
                    let range = open_ctx.range.start..closed_range.end;
                    visitor.on_array_close(range.clone());
                    ArrayState::End(range)
                }
                Some(token_ctx) if token_ctx.token.is_start_of_value() => ArrayState::Value {
                    open_ctx: open_ctx.clone(),
                    expect_ctx: open_ctx.clone(),
                },
                Some(_) => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx.clone(), tok)
                                .expect("array should open with a square bracket")
                        },
                        tokens.next_token()?,
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
                open_ctx,
                expect_ctx,
            } => {
                validate_start_of_value(text, expect_ctx, tokens.peek_token()?.cloned())?;

                let value_range = parse_tokens(tokens, text, false, visitor)?;
                ArrayState::CommaOrEnd {
                    open_ctx,
                    last_value_range: value_range,
                }
            }

            ArrayState::CommaOrEnd { open_ctx, .. } => match tokens.peek_token()?.cloned() {
                Some(TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: closed_range,
                }) => {
                    tokens.next_token()?;

                    let range = open_ctx.range.start..closed_range.end;
                    visitor.on_array_close(range.clone());
                    ArrayState::End(range)
                }
                Some(
                    comma_ctx @ TokenWithContext {
                        token: Token::Comma,
                        ..
                    },
                ) => {
                    tokens.next_token()?;
                    visitor.on_item_delim();
                    ArrayState::Value {
                        open_ctx,
                        expect_ctx: comma_ctx,
                    }
                }
                _ => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx.clone(), tok)
                                .expect("array should open with a square bracket")
                        },
                        tokens.next_token()?,
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

pub fn parse_array<'a, H: ParseVisitor<'a>>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    visitor: &mut H,
) -> Result<'a, Range<usize>> {
    let mut state = ArrayState::Open;

    loop {
        state = state.process(tokens, text, visitor)?;
        if let ArrayState::End(result) = state {
            break Ok(result);
        }
    }
}
