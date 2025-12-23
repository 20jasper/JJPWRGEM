use crate::{
    Result,
    ast::Value,
    format::Emitter,
    tokens::TokenStream,
    traverse::{Visitor, parse_tokens, parse_value},
};
use std::borrow::Cow;

pub fn uglify_str(json: &str) -> Result<'_, String> {
    let mut visitor = UglifyEmitVisitor::default();
    parse_tokens(&mut TokenStream::new(json), json, true, &mut visitor)?;
    Ok(visitor.buf)
}

#[derive(Debug, Default)]
pub struct UglifyEmitVisitor {
    pub buf: String,
}

impl Emitter for UglifyEmitVisitor {
    fn push(&mut self, c: char) {
        self.buf.push(c);
    }

    fn push_str(&mut self, s: &str) {
        self.buf.push_str(s);
    }
}

impl<'a> Visitor<'a> for UglifyEmitVisitor {
    fn on_object_open(&mut self) {
        self.emit_object_open();
    }

    fn on_object_key(&mut self, key: &str) {
        self.emit_string(key);
    }

    fn on_object_key_val_delim(&mut self) {
        self.emit_key_val_delim();
    }

    fn on_object_close(&mut self) {
        self.emit_object_close();
    }

    fn on_array_open(&mut self) {
        self.emit_array_open();
    }

    fn on_array_close(&mut self) {
        self.emit_array_close();
    }

    fn on_null(&mut self) {
        self.emit_null();
    }

    fn on_string(&mut self, s: &str) {
        self.emit_string(s);
    }

    fn on_number(&mut self, n: Cow<'_, str>) {
        self.emit_number(&n);
    }

    fn on_boolean(&mut self, b: bool) {
        self.emit_boolean(b);
    }

    fn on_item_delim(&mut self) {
        self.emit_item_delim();
    }
}

pub fn uglify_value(val: &Value) -> String {
    let mut visitor = UglifyEmitVisitor::default();
    parse_value(val, &mut visitor);
    visitor.buf
}
