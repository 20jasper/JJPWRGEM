use core::range::Range;

use crate::{
    Result,
    ast::Document,
    format::Emitter,
    tokens::TokenStream,
    traverse::{Visitor, parse_tokens, visit_document},
};

pub fn uglify_str(json: &str) -> Result<String> {
    let mut buf = String::new();
    uglify_str_into(&mut buf, json)?;
    Ok(buf)
}

pub fn uglify_str_into(buf: &mut String, json: &str) -> Result<()> {
    let mut visitor = UglifyEmitVisitor {
        buf: std::mem::take(buf),
    };
    parse_tokens(&mut TokenStream::new(json), json, true, &mut visitor)?;
    *buf = visitor.buf;
    Ok(())
}

#[derive(Debug, Default)]
pub(crate) struct UglifyEmitVisitor {
    pub(crate) buf: String,
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
    fn on_object_open(&mut self, _range: Range<usize>) {
        self.emit_object_open();
    }

    fn on_object_key(&mut self, _range: Range<usize>, key: &'a str) {
        self.emit_string(key);
    }

    fn on_object_key_val_delim(&mut self) {
        self.emit_key_val_delim();
    }

    fn on_object_close(&mut self, _range: Range<usize>) {
        self.emit_object_close();
    }

    fn on_array_open(&mut self, _range: Range<usize>) {
        self.emit_array_open();
    }

    fn on_array_close(&mut self, _range: Range<usize>) {
        self.emit_array_close();
    }

    fn on_null(&mut self, _range: Range<usize>) {
        self.emit_null();
    }

    fn on_string(&mut self, _range: Range<usize>, s: &'a str) {
        self.emit_string(s);
    }

    fn on_mantissa(&mut self, _range: Range<usize>, mantissa: &'a str) {
        self.emit_mantissa(mantissa);
    }

    fn on_exponent(&mut self, _range: Range<usize>, exponent: &'a str) {
        self.emit_exponent(exponent);
    }

    fn on_boolean(&mut self, _range: Range<usize>, b: bool) {
        self.emit_boolean(b);
    }

    fn on_item_delim(&mut self) {
        self.emit_item_delim();
    }
}

pub fn uglify_document<S: AsRef<str>>(doc: &Document<S>) -> String {
    let mut visitor = UglifyEmitVisitor::default();
    visit_document(doc, &mut visitor);
    visitor.buf
}

pub fn uglify_document_into<S: AsRef<str>>(buf: &mut String, doc: &Document<S>) {
    let mut visitor = UglifyEmitVisitor {
        buf: std::mem::take(buf),
    };
    visit_document(doc, &mut visitor);
    *buf = visitor.buf;
}
