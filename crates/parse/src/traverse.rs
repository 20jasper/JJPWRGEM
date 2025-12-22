mod array;
mod object;

use crate::{
    Error, ErrorKind, Result,
    tokens::{Token, TokenStream, TokenWithContext},
    traverse::{array::parse_array, object::parse_object},
};
use core::ops::Range;

pub trait ParseVisitor<'a> {
    fn on_object_open(&mut self, open_ctx: TokenWithContext<'a>);
    fn on_object_key(&mut self, key_ctx: TokenWithContext<'a>);
    fn on_object_close(&mut self, range: Range<usize>);
    fn on_array_open(&mut self, open_ctx: TokenWithContext<'a>);
    fn on_array_close(&mut self, range: Range<usize>);
    fn on_scalar(&mut self, token_ctx: TokenWithContext<'a>);
}

pub fn parse_tokens<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    fail_on_multiple_value: bool,
    visitor: &mut impl ParseVisitor<'a>,
) -> Result<'a, Range<usize>> {
    let Some(peeked) = tokens.peek_token()? else {
        return Err(Error::from_maybe_token_with_context(
            |tok| ErrorKind::ExpectedValue(None, tok),
            None,
            text,
        ));
    };
    let range = match &peeked.token {
        Token::OpenCurlyBrace => parse_object(tokens, text, visitor)?,
        Token::OpenSquareBracket => parse_array(tokens, text, visitor)?,
        t if t.is_scalar() => {
            let token_ctx = tokens
                .next_token()?
                .expect("peek guaranteed a value for scalar token");
            visitor.on_scalar(token_ctx.clone());
            token_ctx.range.clone()
        }
        invalid => {
            return Err(Error::new(
                ErrorKind::ExpectedValue(None, Some(invalid.clone()).into()),
                peeked.range.clone(),
                text,
            ));
        }
    };

    if fail_on_multiple_value
        && let Some(TokenWithContext { token, range }) = tokens.peek_token()?
    {
        return Err(Error::new(
            ErrorKind::TokenAfterEnd(token.clone()),
            range.clone(),
            text,
        ));
    }

    Ok(range)
}

fn validate_start_of_value<'a>(
    text: &'a str,
    expect_ctx: TokenWithContext<'a>,
    maybe_token: Option<TokenWithContext<'a>>,
) -> Result<'a, ()> {
    if !maybe_token
        .as_ref()
        .is_some_and(|ctx| ctx.token.is_start_of_value())
    {
        Err(Error::from_maybe_token_with_context(
            |tok| ErrorKind::ExpectedValue(Some(expect_ctx.clone()), tok),
            maybe_token,
            text,
        ))
    } else {
        Ok(())
    }
}
