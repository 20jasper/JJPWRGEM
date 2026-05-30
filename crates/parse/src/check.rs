use core::range::Range;

use crate::{
    Result,
    tokens::TokenStream,
    traverse::{Visitor, parse_tokens},
};

#[derive(Debug)]
pub struct NoopVisitor;

impl<'a> Visitor<'a> for NoopVisitor {
    fn on_object_open(&mut self, _range: Range<usize>) {}
    fn on_object_key(&mut self, _range: Range<usize>, _key: &'a str) {}
    fn on_object_close(&mut self, _range: Range<usize>) {}
    fn on_array_open(&mut self, _range: Range<usize>) {}
    fn on_array_close(&mut self, _range: Range<usize>) {}
    fn on_null(&mut self, _range: Range<usize>) {}
    fn on_string(&mut self, _range: Range<usize>, _value: &'a str) {}
    fn on_mantissa(&mut self, _range: Range<usize>, _mantissa: &'a str) {}
    fn on_exponent(&mut self, _range: Range<usize>, _exponent: &'a str) {}
    fn on_boolean(&mut self, _range: Range<usize>, _value: bool) {}
    fn on_object_key_val_delim(&mut self) {}
    fn on_item_delim(&mut self) {}
}

pub fn validate_str(json: &str) -> Result<()> {
    let mut visitor = NoopVisitor;
    parse_tokens(&mut TokenStream::new(json), json, true, &mut visitor)?;
    Ok(())
}
