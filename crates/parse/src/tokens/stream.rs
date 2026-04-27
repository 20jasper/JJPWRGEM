use core::{iter::Peekable, str::CharIndices};

use crate::{
    Error, ErrorKind, Result,
    tokens::{
        CharWithContext, FALSE, NULL, TRUE, Token, TokenWithContext,
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
    chars: Peekable<CharsWithContext<'a>>,
    input: &'a str,
}

impl<'a> TokenStreamInner<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            chars: CharsWithContext::new(s, 0).peekable(),
            input: s,
        }
    }

    fn consume_whitespace(&mut self) {
        let Some(CharWithContext(r, c)) = self.chars.peek() else {
            return;
        };
        if !c.is_whitespace() {
            return;
        }

        let new_pos = r.start + skip_whitespace(&self.input.as_bytes()[r.start..]);
        self.chars = CharsWithContext::new(&self.input[new_pos..], new_pos).peekable();
    }
}

impl<'a> Iterator for TokenStreamInner<'a> {
    type Item = Result<'a, TokenWithContext<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.consume_whitespace();
            let ctx = self.chars.peek()?;

            let CharWithContext(r, JsonChar(c)) = *ctx;
            if let Some(tok) = ctx.as_token_with_context() {
                self.chars.next();
                return Some(Ok(tok));
            }
            let token = match c {
                '"' => return Some(parse_string(self.input, &mut self.chars)),
                '0'..='9' | '-' => return Some(parse_mantissa(self.input, &mut self.chars)),
                'e' | 'E' => {
                    self.chars.next();
                    match parse_exponent(self.input, r, &mut self.chars) {
                        Ok(Some(tok)) => return Some(Ok(tok)),
                        Ok(None) => continue, // zero exponent stripped skip to next token
                        Err(e) => return Some(Err(e)),
                    }
                }
                'n' | 't' | 'f' => {
                    let expected = match c {
                        'n' => NULL,
                        't' => TRUE,
                        'f' => FALSE,
                        _ => unreachable!("{c} is not able to be reached"),
                    };
                    let actual = self
                        .chars
                        .by_ref()
                        .take(expected.len())
                        .map(|c| c.as_char());

                    if actual.eq(expected.chars()) {
                        let token = match c {
                            'n' => Token::Null,
                            't' => true.into(),
                            'f' => false.into(),
                            _ => unreachable!("{c} is not able to be reached"),
                        };
                        let end = *self
                            .chars
                            .peek()
                            .map(|CharWithContext(r, _)| &r.start)
                            .unwrap_or(&self.input.len());
                        Ok(TokenWithContext {
                            token,
                            range: r.start..end,
                        })
                    } else {
                        Err(Error::new(
                            ErrorKind::UnexpectedCharacter(c.into()),
                            r,
                            self.input,
                        ))
                    }
                }
                _ => Err(Error::new(
                    ErrorKind::UnexpectedCharacter(c.into()),
                    r,
                    self.input,
                )),
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
