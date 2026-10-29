use core::range::Range;

use crate::{
    error::{Error, ErrorKind, Result},
    tokens::{Token, TokenStream, TokenWithContext, byte_range},
    traverse::{Visitor, parse_tokens_at_depth, punctuation_at},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum ArrayState {
    ValueOrEnd,
    Value,
    CommaOrEnd,
    End,
}

impl ArrayState {
    fn process(self, next: Option<Token>) -> Option<Self> {
        let next = next?;
        let state = match self {
            ArrayState::ValueOrEnd if next == Token::ClosedSquareBracket => ArrayState::End,
            ArrayState::ValueOrEnd | ArrayState::Value if next.is_start_of_value() => {
                ArrayState::CommaOrEnd
            }
            ArrayState::CommaOrEnd => match next {
                Token::ClosedSquareBracket => ArrayState::End,
                Token::Comma => ArrayState::Value,
                _ => return None,
            },
            _ => return None,
        };
        Some(state)
    }
}

fn open_ctx(open_start: usize) -> TokenWithContext {
    TokenWithContext::new(Token::OpenSquareBracket, byte_range(open_start))
}

fn array_error(
    state: ArrayState,
    text: &str,
    open_start: usize,
    delim: usize,
    found: Option<TokenWithContext>,
) -> Error {
    match state {
        ArrayState::Value => Error::from_maybe_token_with_context(
            |tok| ErrorKind::ExpectedValue(Some(punctuation_at(text, delim)), tok),
            found,
            text,
        ),
        _ => Error::from_maybe_token_with_context(
            |tok| {
                ErrorKind::expected_entry_or_closed_delimiter(open_ctx(open_start), tok)
                    .expect("array should open with a square bracket")
            },
            found,
            text,
        ),
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
        let peeked = tokens.peek_token()?.copied();
        let Some(next) = state.process(peeked.map(|ctx| ctx.token)) else {
            return Err(array_error(state, text, open_start, delim, peeked));
        };
        let ctx = peeked.expect("the end of input is never accepted");
        match next {
            ArrayState::End => {
                tokens.next_token()?;
                visitor.on_array_close(ctx.range);
            }
            ArrayState::Value => {
                tokens.next_token()?;
                visitor.on_item_delim();
                delim = ctx.range.start;
            }
            ArrayState::CommaOrEnd => {
                parse_tokens_at_depth(tokens, text, false, visitor, depth + 1)?;
            }
            ArrayState::ValueOrEnd => unreachable!("only the starting state"),
        }
        state = next;
    }

    // the cursor sits right after the closing `]`
    Ok(open_start..tokens.pos())
}
