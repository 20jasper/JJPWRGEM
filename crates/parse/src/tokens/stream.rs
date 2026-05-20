use core::iter::Peekable;

use crate::{
    Error, ErrorKind, Result,
    tokens::{
        ByteWithContext, BytesWithContext, FALSE, NULL, TRUE, Token, TokenWithContext,
        current_byte_pos,
        lexical::JsonChar,
        number::{parse_exponent, parse_mantissa},
        string::parse_string,
    },
};

fn skip_whitespace(bytes: &[u8]) -> usize {
    use std::simd::{cmp::SimdPartialEq as _, u8x32};

    let space = u8x32::splat(b' ');
    let horizontal_tab = u8x32::splat(b'\t');
    let line_feed = u8x32::splat(b'\n');
    let carriage_return = u8x32::splat(b'\r');

    let mut i = 0;
    while i + 32 <= bytes.len() {
        let chunk = u8x32::from_slice(&bytes[i..i + 32]);
        let is_whitespace = chunk.simd_eq(space)
            | chunk.simd_eq(horizontal_tab)
            | chunk.simd_eq(line_feed)
            | chunk.simd_eq(carriage_return);
        let whitespace_count = is_whitespace.to_bitmask().trailing_ones() as usize;
        i += whitespace_count;
        if whitespace_count < 32 {
            return i;
        }
    }

    i + bytes[i..]
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        .unwrap_or(bytes.len().saturating_sub(i))
}

#[derive(Debug, Clone)]
struct TokenStreamInner<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> TokenStreamInner<'a> {
    fn new(s: &'a str) -> Self {
        Self { input: s, pos: 0 }
    }

    fn peek_byte_with_context(&self) -> Option<ByteWithContext> {
        self.input
            .as_bytes()
            .get(self.pos)
            .copied()
            .map(|byte| (self.pos, byte).into())
    }

    fn bytes_from_pos(&self) -> Peekable<BytesWithContext<'a>> {
        BytesWithContext::new(self.input, self.pos).peekable()
    }

    fn parse_with_bytes<T>(
        &mut self,
        f: impl FnOnce(&'a str, &mut Peekable<BytesWithContext<'a>>) -> Result<T>,
    ) -> Result<T> {
        let mut bytes = self.bytes_from_pos();
        let result = f(self.input, &mut bytes);
        if result.is_ok() {
            self.update_pos_from_bytes(&mut bytes);
        }
        result
    }

    fn update_pos_from_bytes(&mut self, bytes: &mut Peekable<BytesWithContext<'a>>) {
        self.pos = current_byte_pos(bytes, self.input);
    }

    fn unexpected_character(&self) -> Result<TokenWithContext> {
        let unexpected = self.input[self.pos..]
            .chars()
            .next()
            .expect("pos must be in bounds");
        Err(Error::new(
            ErrorKind::UnexpectedCharacter(JsonChar(unexpected)),
            self.pos..self.pos + unexpected.len_utf8(),
            self.input,
        ))
    }

    fn consume_whitespace(&mut self) {
        let Some(ByteWithContext(_, byte)) = self.peek_byte_with_context() else {
            return;
        };
        if !byte.is_whitespace() {
            return;
        }

        self.pos += skip_whitespace(&self.input.as_bytes()[self.pos..]);
    }
}

impl Iterator for TokenStreamInner<'_> {
    type Item = Result<TokenWithContext>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.consume_whitespace();
            let ctx = self.peek_byte_with_context()?;

            if let Some(tok) = ctx.as_token_with_context() {
                self.pos = ctx.range().end;
                return Some(Ok(tok));
            }

            let token = match ctx.as_byte() {
                b'"' => {
                    let result = parse_string(self.input, self.pos);
                    if let Ok(token) = &result {
                        self.pos = token.range.end;
                    }
                    result
                }
                b'0'..=b'9' | b'-' => self.parse_with_bytes(parse_mantissa),
                b'e' | b'E' => {
                    match self.parse_with_bytes(|input, bytes| {
                        bytes.next();
                        parse_exponent(input, ctx.range(), bytes)
                    }) {
                        Ok(Some(tok)) => Ok(tok),
                        Ok(None) => continue,
                        Err(err) => Err(err),
                    }
                }
                b'n' | b't' | b'f' => {
                    let (expected, token) = match ctx.as_byte() {
                        b'n' => (NULL, Token::Null),
                        b't' => (TRUE, Token::True),
                        b'f' => (FALSE, Token::False),
                        _ => unreachable!("matched above"),
                    };
                    let end = self.pos + expected.len();
                    if self.input.as_bytes().get(self.pos..end) == Some(expected.as_bytes()) {
                        self.pos = end;
                        Ok(TokenWithContext::new(token, ctx.0..end))
                    } else {
                        self.unexpected_character()
                    }
                }
                _ => self.unexpected_character(),
            };

            return Some(token);
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    inner: TokenStreamInner<'a>,
    cached: Option<TokenWithContext>,
}

impl<'a> TokenStream<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            inner: TokenStreamInner::new(s),
            cached: None,
        }
    }

    pub fn peek_token(&mut self) -> Result<Option<&TokenWithContext>> {
        if self.cached.is_none() {
            match self.inner.next() {
                Some(Ok(token)) => self.cached = Some(token),
                Some(Err(err)) => return Err(err),
                None => return Ok(None),
            }
        }

        Ok(self.cached.as_ref())
    }

    pub fn next_token(&mut self) -> Result<Option<TokenWithContext>> {
        if let Some(token) = self.cached.take() {
            Ok(Some(token))
        } else {
            self.next().transpose()
        }
    }
}

impl Iterator for TokenStream<'_> {
    type Item = Result<TokenWithContext>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
