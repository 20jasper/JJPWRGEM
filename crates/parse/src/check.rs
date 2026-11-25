use core::range::Range;

use crate::{
    Result, SyntaxOptions,
    settings::{choice, choice_enum, settings},
    tokens::TokenStream,
    traverse::{Visitor, parse_tokens},
};

choice_enum! {
    #[cfg_attr(feature = "serde", derive(::serde::Deserialize), serde(rename_all = "lowercase"))]
    pub enum CheckParser default Json {
        /// A single json document
        Json = "json",
        /// A single json document with comments and trailing commas
        Jsonc = "jsonc",
        /// One json document per line
        Jsonlines = "jsonlines",
    }
}

settings! {
    #[cfg_attr(feature = "serde", derive(::serde::Deserialize), serde(rename_all = "camelCase", default))]
    struct CheckInputSettings, table CHECK_INPUT_SETTINGS, id CheckInputSetting {
        /// Parser to use for input
        Parser parser: CheckParser = choice(),
    }
}

#[derive(Debug)]
pub struct NoopVisitor;

impl<'a> Visitor<'a> for NoopVisitor {
    #[inline]
    fn on_object_open(&mut self, _range: Range<usize>) {}
    #[inline]
    fn on_object_key(&mut self, _range: Range<usize>, _key: &'a str) {}
    #[inline]
    fn on_object_close(&mut self, _range: Range<usize>) {}
    #[inline]
    fn on_array_open(&mut self, _range: Range<usize>) {}
    #[inline]
    fn on_array_close(&mut self, _range: Range<usize>) {}
    #[inline]
    fn on_null(&mut self, _range: Range<usize>) {}
    #[inline]
    fn on_string(&mut self, _range: Range<usize>, _value: &'a str) {}
    #[inline]
    fn on_mantissa(&mut self, _range: Range<usize>, _mantissa: &'a str) {}
    #[inline]
    fn on_exponent(&mut self, _range: Range<usize>, _exponent: &'a str) {}
    #[inline]
    fn on_boolean(&mut self, _range: Range<usize>, _value: bool) {}
    #[inline]
    fn on_object_key_val_delim(&mut self) {}
    #[inline]
    fn on_item_delim(&mut self) {}
}

pub fn validate_str(json: &str, options: SyntaxOptions) -> Result<()> {
    let mut visitor = NoopVisitor;
    parse_tokens(
        &mut TokenStream::new(json, options),
        options,
        json,
        true,
        &mut visitor,
    )?;
    Ok(())
}
