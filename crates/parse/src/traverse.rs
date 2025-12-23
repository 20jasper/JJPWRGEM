mod array;
mod object;

pub use parse::{ParseVisitor, parse_tokens, validate_start_of_value};
pub use value::{ValueVisitor, parse_value};
mod parse {
    use crate::{
        Error, ErrorKind, Result,
        tokens::{Token, TokenStream, TokenWithContext},
        traverse::{array::parse_array, object::parse_object},
    };
    use core::ops::Range;
    use std::borrow::Cow;

    pub trait ParseVisitor<'a> {
        fn on_object_open(&mut self, open_ctx: TokenWithContext<'a>);
        fn on_object_key(&mut self, key: &'a str);
        fn on_object_key_val_delim(&mut self);
        fn on_object_close(&mut self, range: Range<usize>);
        fn on_array_open(&mut self, open_ctx: TokenWithContext<'a>);
        fn on_array_close(&mut self, range: Range<usize>);
        fn on_null(&mut self);
        fn on_string(&mut self, value: &'a str);
        fn on_number(&mut self, value: Cow<'a, str>);
        fn on_boolean(&mut self, value: bool);
        fn on_item_delim(&mut self);
    }

    pub fn parse_tokens<'a>(
        tokens: &mut TokenStream<'a>,
        text: &'a str,
        fail_on_multiple_value: bool,
        visitor: &mut impl ParseVisitor<'a>,
    ) -> Result<'a, Range<usize>> {
        let peeked = tokens.peek_token()?.cloned();
        let Some(peeked) = peeked else {
            return Err(Error::from_maybe_token_with_context(
                |tok| ErrorKind::ExpectedValue(None, tok),
                None,
                text,
            ));
        };
        let range = match peeked.token {
            Token::OpenCurlyBrace => parse_object(tokens, text, visitor)?,
            Token::OpenSquareBracket => parse_array(tokens, text, visitor)?,
            t if t.is_scalar() => {
                let token_ctx = tokens
                    .next_token()?
                    .expect("peek guaranteed a value for scalar token");

                match t {
                    Token::String(s) => visitor.on_string(s),
                    Token::Number(cow) => visitor.on_number(cow),
                    Token::Null => visitor.on_null(),
                    Token::Boolean(b) => visitor.on_boolean(b),
                    _ => unreachable!("guard prevents non scalars"),
                };

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

    pub fn validate_start_of_value<'a>(
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
}

mod value {
    use crate::ast::{ObjectEntries, Value};

    pub trait ValueVisitor<'a> {
        fn on_null(&mut self);
        fn on_string(&mut self, value: &str);
        fn on_number(&mut self, value: &str);
        fn on_boolean(&mut self, value: bool);
        fn on_array_open(&mut self);
        fn on_array_close(&mut self);
        fn on_object_open(&mut self);
        fn on_object_key_val_delim(&mut self);
        fn on_object_close(&mut self);
        fn on_item_delim(&mut self);
    }

    fn join<V, T>(
        visitor: &mut V,
        items: impl IntoIterator<Item = T>,
        mut item_fmt: impl FnMut(&mut V, &T),
        mut delim_fmt: impl FnMut(&mut V, &T),
    ) {
        let mut iter = items.into_iter();
        if let Some(first) = iter.next() {
            item_fmt(visitor, &first);
            for item in iter {
                delim_fmt(visitor, &item);
                item_fmt(visitor, &item);
            }
        }
    }

    pub fn parse_value<'a>(val: &Value, visitor: &mut impl ValueVisitor<'a>) {
        match val {
            Value::Null => visitor.on_null(),
            Value::String(s) => visitor.on_string(s),
            Value::Number(n) => visitor.on_number(n),
            Value::Boolean(b) => visitor.on_boolean(*b),
            Value::Object(ObjectEntries(items)) => {
                visitor.on_object_open();
                join(
                    visitor,
                    items,
                    |visitor, (k, v)| {
                        visitor.on_string(k);
                        visitor.on_object_key_val_delim();
                        parse_value(v, visitor);
                    },
                    |visitor, _| visitor.on_item_delim(),
                );
                visitor.on_object_close();
            }
            Value::Array(items) => {
                visitor.on_array_open();
                join(
                    visitor,
                    items,
                    |visitor, val| {
                        parse_value(val, visitor);
                    },
                    |visitor, _| visitor.on_item_delim(),
                );
                visitor.on_array_close();
            }
        };
    }
}
