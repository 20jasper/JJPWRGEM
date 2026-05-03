use core::{iter::Peekable, str::CharIndices};

use crate::{
    Error, ErrorKind, Result,
    tokens::{
        ByteWithContext, CharWithContext, FALSE, NULL, TRUE, Token, TokenWithContext,
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
struct CharsWithContext<'a> {
    iter: CharIndices<'a>,
    offset: usize,
}

impl<'a> CharsWithContext<'a> {
    fn new(s: &'a str, offset: usize) -> Self {
        Self {
            iter: s.char_indices(),
            offset,
        }
    }
}

impl<'a> Iterator for CharsWithContext<'a> {
    type Item = CharWithContext;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(i, c)| (i + self.offset, c).into())
    }
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

    fn chars_from_pos(&self) -> Peekable<CharsWithContext<'a>> {
        CharsWithContext::new(&self.input[self.pos..], self.pos).peekable()
    }

    fn parse_with_chars<T>(
        &mut self,
        f: impl FnOnce(&'a str, &mut Peekable<CharsWithContext<'a>>) -> Result<'a, T>,
    ) -> Result<'a, T> {
        let mut chars = self.chars_from_pos();
        let result = f(self.input, &mut chars);
        if result.is_ok() {
            self.update_pos_from_chars(&mut chars);
        }
        result
    }

    fn update_pos_from_chars(&mut self, chars: &mut Peekable<CharsWithContext<'a>>) {
        self.pos = chars
            .peek()
            .map(|CharWithContext(range, _)| range.start)
            .unwrap_or(self.input.len());
    }

    fn unexpected_character(&self) -> Result<'a, TokenWithContext<'a>> {
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

impl<'a> Iterator for TokenStreamInner<'a> {
    type Item = Result<'a, TokenWithContext<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.consume_whitespace();
            let ctx = self.peek_byte_with_context()?;

            if let Some(tok) = ctx.as_token_with_context() {
                self.pos = ctx.range().end;
                return Some(Ok(tok));
            }

            let token = match ctx.as_byte() {
                b'"' => self.parse_with_chars(parse_string),
                b'0'..=b'9' | b'-' => self.parse_with_chars(parse_mantissa),
                b'e' | b'E' => {
                    match self.parse_with_chars(|input, chars| {
                        chars.next();
                        parse_exponent(input, ctx.range(), chars)
                    }) {
                        Ok(Some(tok)) => Ok(tok),
                        Ok(None) => continue,
                        Err(err) => Err(err),
                    }
                }
                b'n' | b't' | b'f' => {
                    let expected = match ctx.as_byte() {
                        b'n' => NULL,
                        b't' => TRUE,
                        b'f' => FALSE,
                        _ => unreachable!("matched above"),
                    };
                    let end = self.pos + expected.len();
                    if self.input.as_bytes().get(self.pos..end) == Some(expected.as_bytes()) {
                        let token = match ctx.as_byte() {
                            b'n' => Token::Null,
                            b't' => true.into(),
                            b'f' => false.into(),
                            _ => unreachable!("matched above"),
                        };
                        self.pos = end;
                        Ok(TokenWithContext {
                            token,
                            range: ctx.0..end,
                        })
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
    cached: Option<TokenWithContext<'a>>,
}

impl<'a> TokenStream<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            inner: TokenStreamInner::new(s),
            cached: None,
        }
    }

    pub fn peek_token(&mut self) -> Result<'a, Option<&TokenWithContext<'a>>> {
        if self.cached.is_none() {
            match self.inner.next() {
                Some(Ok(token)) => self.cached = Some(token),
                Some(Err(err)) => return Err(err),
                None => return Ok(None),
            }
        }

        Ok(self.cached.as_ref())
    }

    pub fn next_token(&mut self) -> Result<'a, Option<TokenWithContext<'a>>> {
        if let Some(token) = self.cached.take() {
            Ok(Some(token))
        } else {
            self.next().transpose()
        }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Result<'a, TokenWithContext<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
