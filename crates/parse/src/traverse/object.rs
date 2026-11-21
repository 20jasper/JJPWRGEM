use core::range::Range;

use crate::{
    Error, ErrorKind, Result,
    tokens::{Token, TokenStream, TokenWithContext, byte_range},
    traverse::{Visitor, parse_tokens_at_depth, punctuation_at},
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
    /// where a comma leads: trailing commas allow the closing brace right after it
    const fn after_comma(trailing_commas: bool) -> Self {
        if trailing_commas {
            Self::KeyOrEnd
        } else {
            Self::Key
        }
    }

    fn process(self, next: Option<Token>, after_comma: Self) -> Option<Self> {
        let next = next?;
        let state = match self {
            Self::KeyOrEnd | Self::CommaOrEnd if next == Token::ClosedCurlyBrace => Self::End,
            Self::KeyOrEnd | Self::Key if next == Token::String => Self::Colon,
            Self::CommaOrEnd if next == Token::Comma => after_comma,
            Self::Colon if next == Token::Colon => Self::Value,
            Self::Value if next.is_start_of_value() => Self::CommaOrEnd,
            _ => return None,
        };
        Some(state)
    }
}

fn object_error(
    state: ObjectState,
    text: &str,
    offsets: &ObjectOffsets,
    found: Option<TokenWithContext>,
) -> Error {
    let open = open_ctx(offsets.open_start);
    Error::from_maybe_token_with_context(
        |tok| match state {
            ObjectState::KeyOrEnd => ErrorKind::expected_entry_or_closed_delimiter(open, tok)
                .expect("object should open with a curly brace"),
            ObjectState::CommaOrEnd => ErrorKind::ExpectedCommaOrClosedCurlyBrace {
                range: offsets.last_pair,
                open_ctx: open,
                found: tok,
            },
            ObjectState::Key => ErrorKind::ExpectedKey(punctuation_at(text, offsets.delim), tok),
            ObjectState::Colon => {
                ErrorKind::ExpectedColon(TokenWithContext::new(Token::String, offsets.key), tok)
            }
            ObjectState::Value => {
                ErrorKind::ExpectedValue(Some(punctuation_at(text, offsets.last_pair.start)), tok)
            }
            ObjectState::End => unreachable!("an object is done once it ends"),
        },
        found,
        text,
    )
}

pub fn parse_object<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    depth: usize,
    visitor: &mut impl Visitor<'a>,
) -> Result<Range<usize>> {
    let Some(TokenWithContext {
        token: Token::OpenCurlyBrace,
        range: open_range,
    }) = tokens.next_token()?
    else {
        unreachable!("callers only parse an object after seeing its `{{`");
    };
    visitor.on_object_open(open_range);
    let open_start = open_range.start;

    let mut offsets = ObjectOffsets {
        open_start,
        delim: open_start,
        key: open_start..open_start,
        last_pair: open_start..open_start,
    };
    let after_comma = ObjectState::after_comma(tokens.options().trailing_commas);
    let mut state = ObjectState::KeyOrEnd;
    while state != ObjectState::End {
        // a value is parsed from its first token, everything else is taken right away
        let found = if state == ObjectState::Value {
            tokens.peek_token()?.copied()
        } else {
            tokens.next_token()?
        };
        let Some(next) = state.process(found.map(|ctx| ctx.token), after_comma) else {
            return Err(object_error(state, text, &offsets, found));
        };
        let ctx = found.expect("the end of input is never accepted");
        match next {
            ObjectState::End => visitor.on_object_close(ctx.range),
            ObjectState::Key | ObjectState::KeyOrEnd => {
                offsets.delim = ctx.range.start;
            }
            ObjectState::Colon => {
                if offsets.delim != offsets.open_start {
                    visitor.on_item_delim();
                }
                let body = ctx.content_range();
                visitor.on_object_key(body, &text[body]);
                offsets.key = ctx.range;
            }
            ObjectState::Value => {
                visitor.on_object_key_val_delim();
                offsets.last_pair = ctx.range;
            }
            ObjectState::CommaOrEnd => {
                let value_range = parse_tokens_at_depth(tokens, text, false, visitor, depth + 1)?;
                offsets.last_pair = offsets.last_pair.start..value_range.end;
            }
        }
        state = next;
    }

    // the cursor sits right after the closing `}`
    Ok(open_start..tokens.pos())
}
