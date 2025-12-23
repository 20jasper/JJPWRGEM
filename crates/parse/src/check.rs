use std::borrow;

use crate::{
    Result,
    tokens::TokenStream,
    traverse::{Visitor, parse_tokens},
};

#[derive(Debug)]
pub struct NoopVisitor;

impl<'a> Visitor<'a> for NoopVisitor {
    fn on_object_open(&mut self) {}
    fn on_object_key(&mut self, _key: &'a str) {}
    fn on_object_close(&mut self) {}
    fn on_array_open(&mut self) {}
    fn on_array_close(&mut self) {}
    fn on_null(&mut self) {}
    fn on_string(&mut self, _value: &'a str) {}
    fn on_number(&mut self, _value: borrow::Cow<'a, str>) {}
    fn on_boolean(&mut self, _value: bool) {}
    fn on_object_key_val_delim(&mut self) {}
    fn on_item_delim(&mut self) {}
}

pub fn validate_str<'a>(json: &'a str) -> Result<'a, ()> {
    let mut visitor = NoopVisitor;
    parse_tokens(&mut TokenStream::new(json), json, true, &mut visitor)?;
    Ok(())
}
