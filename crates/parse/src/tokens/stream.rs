use core::{iter::Peekable, str::CharIndices};

use crate::tokens::{
    CharWithContext, Token, TokenWithContext, lexical::JsonChar, number::parse_num,
    string::parse_string,
};
use crate::tokens::{FALSE, NULL, TRUE};
use crate::{Error, ErrorKind, Result};

#[derive(Debug, Clone)]
struct CharsWithContext<'a> {
    iter: CharIndices<'a>,
}

impl<'a> CharsWithContext<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            iter: s.char_indices(),
        }
    }
}

impl<'a> Iterator for CharsWithContext<'a> {
    type Item = CharWithContext;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Into::into)
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
            chars: CharsWithContext::new(s).peekable(),
            input: s,
        }
    }

    fn consume_whitespace(&mut self) {
        while self
            .chars
            .next_if(|CharWithContext(_, x)| x.is_whitespace())
            .is_some()
        {}
    }
}

impl<'a> Iterator for TokenStreamInner<'a> {
    type Item = Result<'a, TokenWithContext<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();
        let ctx = self.chars.peek()?;

        let CharWithContext(r, JsonChar(c)) = ctx.clone();
        if let Some(tok) = ctx.as_token_with_context() {
            self.chars.next();
            return Some(Ok(tok));
        }
        let token = match c {
            '"' => parse_string(self.input, &mut self.chars),
            '0'..='9' | '-' => parse_num(self.input, &mut self.chars),
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
                        r.clone(),
                        self.input,
                    ))
                }
            }
            _ => Err(Error::new(
                ErrorKind::UnexpectedCharacter(c.into()),
                r.clone(),
                self.input,
            )),
        };

        Some(token)
    }
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    inner: Peekable<TokenStreamInner<'a>>,
}

impl<'a> TokenStream<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            inner: TokenStreamInner::new(s).peekable(),
        }
    }

    pub fn peek_token(&mut self) -> Result<'a, Option<TokenWithContext<'a>>> {
        self.inner.peek().cloned().transpose()
    }

    pub fn next_token(&mut self) -> Result<'a, Option<TokenWithContext<'a>>> {
        self.inner.next().transpose()
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Result<'a, TokenWithContext<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
