mod array;
mod object;

use core::range::Range;

use crate::{
    Error, ErrorKind, Result,
    tokens::{ErrorToken, Token, TokenStream, TokenWithContext},
    traverse::{array::parse_array, object::parse_object},
};

pub trait Visitor<'a> {
    fn on_object_open(&mut self, range: Range<usize>);
    fn on_object_key(&mut self, range: Range<usize>, key: &'a str);
    fn on_object_key_val_delim(&mut self);
    fn on_object_close(&mut self, range: Range<usize>);
    fn on_array_open(&mut self, range: Range<usize>);
    fn on_array_close(&mut self, range: Range<usize>);
    fn on_null(&mut self, range: Range<usize>);
    fn on_string(&mut self, range: Range<usize>, value: &'a str);
    fn on_mantissa(&mut self, range: Range<usize>, mantissa: &'a str);
    fn on_exponent(&mut self, range: Range<usize>, exponent: &'a str);
    fn on_boolean(&mut self, range: Range<usize>, value: bool);
    fn on_item_delim(&mut self);
}

const MAX_DEPTH: usize = 128;

pub fn parse_tokens<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    fail_on_multiple_value: bool,
    visitor: &mut impl Visitor<'a>,
) -> Result<Range<usize>> {
    parse_tokens_at_depth(tokens, text, fail_on_multiple_value, visitor, 0)
}

fn parse_tokens_at_depth<'a>(
    tokens: &mut TokenStream<'a>,
    text: &'a str,
    fail_on_multiple_value: bool,
    visitor: &mut impl Visitor<'a>,
    depth: usize,
) -> Result<Range<usize>> {
    if depth >= MAX_DEPTH {
        return Err(Error::new(
            ErrorKind::NestingTooDeep(MAX_DEPTH),
            0..text.len(),
            text,
        ));
    }
    let peeked = tokens.peek_token()?.copied();
    let Some(peeked) = peeked else {
        let pos = tokens.pos();
        return Err(Error::new(
            ErrorKind::ExpectedValue(None, None.into()),
            pos..pos,
            text,
        ));
    };
    let range = match peeked.token {
        Token::OpenCurlyBrace => parse_object(tokens, text, depth, visitor)?,
        Token::OpenSquareBracket => parse_array(tokens, text, depth, visitor)?,
        t if t.is_scalar() => {
            let token_ctx = tokens
                .next_token()?
                .expect("peek guaranteed a value for scalar token");
            let range = token_ctx.range;

            match t {
                Token::String => {
                    let body = token_ctx.content_range();
                    visitor.on_string(body, &text[body]);
                }
                Token::Mantissa => {
                    visitor.on_mantissa(range, &text[range]);
                    if let Some(TokenWithContext {
                        token: Token::Exponent,
                        range: er,
                    }) = tokens.peek_token()?.copied()
                    {
                        let exp_ctx = tokens.next_token()?.expect("peek guaranteed");
                        debug_assert_eq!(
                            exp_ctx.range, er,
                            "peeked exponent range must match the consumed token"
                        );
                        visitor.on_exponent(er, &text[er]);
                    }
                }
                Token::Null => visitor.on_null(range),
                Token::True => visitor.on_boolean(range, true),
                Token::False => visitor.on_boolean(range, false),
                _ => unreachable!("guard prevents non scalars"),
            }

            range
        }
        t => {
            return Err(Error::new(
                ErrorKind::ExpectedValue(None, Some(t).into()),
                peeked.range,
                text,
            ));
        }
    };

    if fail_on_multiple_value && let Some(twc) = tokens.peek_token()?.copied() {
        return Err(Error::new(
            ErrorKind::TokenAfterEnd(ErrorToken::new(twc.token, twc.range, text)),
            twc.range,
            text,
        ));
    }

    Ok(range)
}

pub(crate) fn validate_start_of_value(
    text: &str,
    expect_ctx: TokenWithContext,
    maybe_token: Option<TokenWithContext>,
) -> Result<()> {
    if !maybe_token.is_some_and(|ctx| ctx.token.is_start_of_value()) {
        Err(Error::from_maybe_token_with_context(
            |tok| ErrorKind::ExpectedValue(Some(expect_ctx), tok),
            maybe_token,
            text,
        ))
    } else {
        Ok(())
    }
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

pub fn visit_document<'a, S: AsRef<str>>(
    doc: &'a crate::ast::Document<S>,
    visitor: &mut impl Visitor<'a>,
) {
    visit_value(doc.source.as_ref(), doc.root(), visitor);
}

pub(crate) fn visit_value<'a>(
    source: &'a str,
    val: &'a crate::ast::Value,
    visitor: &mut impl Visitor<'a>,
) {
    use crate::ast::{ObjectEntries, Value};
    match val {
        Value::Null(r) => visitor.on_null(*r),
        Value::String(r) => visitor.on_string(*r, &source[*r]),
        Value::Number { mantissa, exponent } => {
            visitor.on_mantissa(*mantissa, &source[*mantissa]);
            if let Some(exponent) = exponent {
                visitor.on_exponent(*exponent, &source[*exponent]);
            }
        }
        Value::Boolean(r, b) => visitor.on_boolean(*r, *b),
        Value::Object(r, ObjectEntries(items)) => {
            visitor.on_object_open(r.start..r.start + 1);
            join(
                visitor,
                items,
                |visitor, (kr, v)| {
                    visitor.on_object_key(*kr, &source[*kr]);
                    visitor.on_object_key_val_delim();
                    visit_value(source, v, visitor);
                },
                |visitor, _| visitor.on_item_delim(),
            );
            visitor.on_object_close(r.end - 1..r.end);
        }
        Value::Array(r, items) => {
            visitor.on_array_open(r.start..r.start + 1);
            join(
                visitor,
                items,
                |visitor, val| visit_value(source, val, visitor),
                |visitor, _| visitor.on_item_delim(),
            );
            visitor.on_array_close(r.end - 1..r.end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Document;

    #[derive(Debug, PartialEq, Eq)]
    enum Event {
        ObjectOpen(Range<usize>),
        ObjectKey(Range<usize>),
        ObjectKeyValDelim,
        ObjectClose(Range<usize>),
        ArrayOpen(Range<usize>),
        ArrayClose(Range<usize>),
        Null(Range<usize>),
        String(Range<usize>),
        Mantissa(Range<usize>),
        Exponent(Range<usize>),
        Boolean(Range<usize>, bool),
        ItemDelim,
    }

    #[derive(Default)]
    struct RecordingVisitor {
        events: Vec<Event>,
    }

    impl<'a> Visitor<'a> for RecordingVisitor {
        fn on_object_open(&mut self, range: Range<usize>) {
            self.events.push(Event::ObjectOpen(range));
        }

        fn on_object_key(&mut self, range: Range<usize>, _key: &'a str) {
            self.events.push(Event::ObjectKey(range));
        }

        fn on_object_key_val_delim(&mut self) {
            self.events.push(Event::ObjectKeyValDelim);
        }

        fn on_object_close(&mut self, range: Range<usize>) {
            self.events.push(Event::ObjectClose(range));
        }

        fn on_array_open(&mut self, range: Range<usize>) {
            self.events.push(Event::ArrayOpen(range));
        }

        fn on_array_close(&mut self, range: Range<usize>) {
            self.events.push(Event::ArrayClose(range));
        }

        fn on_null(&mut self, range: Range<usize>) {
            self.events.push(Event::Null(range));
        }

        fn on_string(&mut self, range: Range<usize>, _value: &'a str) {
            self.events.push(Event::String(range));
        }

        fn on_mantissa(&mut self, range: Range<usize>, _mantissa: &'a str) {
            self.events.push(Event::Mantissa(range));
        }

        fn on_exponent(&mut self, range: Range<usize>, _exponent: &'a str) {
            self.events.push(Event::Exponent(range));
        }

        fn on_boolean(&mut self, range: Range<usize>, value: bool) {
            self.events.push(Event::Boolean(range, value));
        }

        fn on_item_delim(&mut self) {
            self.events.push(Event::ItemDelim);
        }
    }

    #[test]
    fn visit_document_matches_token_traversal_events() {
        let json = r#"{"a":["b",{"c":1e5}],"d":true}"#;
        let expected = vec![
            Event::ObjectOpen(0..1),
            Event::ObjectKey(2..3),
            Event::ObjectKeyValDelim,
            Event::ArrayOpen(5..6),
            Event::String(7..8),
            Event::ItemDelim,
            Event::ObjectOpen(10..11),
            Event::ObjectKey(12..13),
            Event::ObjectKeyValDelim,
            Event::Mantissa(15..16),
            Event::Exponent(17..18),
            Event::ObjectClose(18..19),
            Event::ArrayClose(19..20),
            Event::ItemDelim,
            Event::ObjectKey(22..23),
            Event::ObjectKeyValDelim,
            Event::Boolean(25..29, true),
            Event::ObjectClose(29..30),
        ];

        let mut from_tokens: RecordingVisitor = RecordingVisitor::default();
        parse_tokens(&mut TokenStream::new(json), json, true, &mut from_tokens).unwrap();

        let doc = Document::parse(json).unwrap();
        let mut from_ast = RecordingVisitor::default();
        visit_document(&doc, &mut from_ast);

        assert_eq!(from_tokens.events, expected);
        assert_eq!(from_ast.events, expected);
        assert_eq!(from_tokens.events, from_ast.events);
    }
}
