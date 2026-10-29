use core::range::Range;

use crate::{
    error::{Error, ErrorKind, Result},
    tokens::{Token, TokenStream, TokenWithContext, byte_range},
    traverse::{Visitor, parse_tokens_at_depth, punctuation_at, validate_start_of_value},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum ArrayState {
    ValueOrEnd,
    Value,
    CommaOrEnd,
    End,
}

fn open_ctx(open_start: usize) -> TokenWithContext {
    TokenWithContext::new(Token::OpenSquareBracket, byte_range(open_start))
}

impl ArrayState {
    fn process<'a>(
        self,
        tokens: &mut TokenStream<'a>,
        text: &'a str,
        visitor: &mut impl Visitor<'a>,
        depth: usize,
        open_start: usize,
        delim: &mut usize,
    ) -> Result<Self> {
        let next_state = match self {
            ArrayState::ValueOrEnd => match tokens.peek_token()?.copied() {
                Some(TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: closed_range,
                }) => {
                    tokens.next_token()?;
                    visitor.on_array_close(closed_range);
                    ArrayState::End
                }
                Some(token_ctx) if token_ctx.token.is_start_of_value() => {
                    parse_tokens_at_depth(tokens, text, false, visitor, depth + 1)?;
                    ArrayState::CommaOrEnd
                }
                Some(_) => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx(open_start), tok)
                                .expect("array should open with a square bracket")
                        },
                        tokens.next_token()?,
                        text,
                    ));
                }
                None => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx(open_start), tok)
                                .expect("array should open with a square bracket")
                        },
                        None,
                        text,
                    ));
                }
            },

            ArrayState::Value => {
                let peeked = tokens.peek_token()?.copied();
                validate_start_of_value(text, || punctuation_at(text, *delim), peeked)?;

                parse_tokens_at_depth(tokens, text, false, visitor, depth + 1)?;
                ArrayState::CommaOrEnd
            }

            ArrayState::CommaOrEnd => match tokens.peek_token()?.copied() {
                Some(TokenWithContext {
                    token: Token::ClosedSquareBracket,
                    range: closed_range,
                }) => {
                    tokens.next_token()?;
                    visitor.on_array_close(closed_range);
                    ArrayState::End
                }
                Some(TokenWithContext {
                    token: Token::Comma,
                    range: comma_range,
                }) => {
                    tokens.next_token()?;
                    visitor.on_item_delim();
                    *delim = comma_range.start;
                    ArrayState::Value
                }
                _ => {
                    return Err(Error::from_maybe_token_with_context(
                        |tok| {
                            ErrorKind::expected_entry_or_closed_delimiter(open_ctx(open_start), tok)
                                .expect("array should open with a square bracket")
                        },
                        tokens.next_token()?,
                        text,
                    ));
                }
            },

            ArrayState::End => self,
        };

        Ok(next_state)
    }
}

pub fn parse_array<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    depth: usize,
    visitor: &mut impl Visitor<'a>,
) -> Result<Range<usize>> {
    let open_start = match tokens.next_token()? {
        Some(
            open_ctx @ TokenWithContext {
                token: Token::OpenSquareBracket,
                ..
            },
        ) => {
            visitor.on_array_open(open_ctx.range);
            open_ctx.range.start
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
    };

    let mut delim = open_start;
    let mut state = ArrayState::ValueOrEnd;
    while state != ArrayState::End {
        state = state.process(tokens, text, visitor, depth, open_start, &mut delim)?;
    }

    // the cursor sits right after the closing `]`
    Ok(open_start..tokens.pos())
}
