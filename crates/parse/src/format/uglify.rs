use crate::{
    Result,
    ast::Value,
    tokens::{FALSE, NULL, TRUE, TokenStream},
    traverse::{ParseVisitor, ValueVisitor, parse_tokens, parse_value},
};

pub fn uglify_str(json: &str) -> Result<'_, String> {
    let mut visitor = UglifyEmitVisitor::default();
    parse_tokens(&mut TokenStream::new(json), json, true, &mut visitor)?;
    Ok(visitor.buf)
}

#[derive(Debug, Default)]
pub struct UglifyEmitVisitor {
    pub buf: String,
}

fn push_quoted(buf: &mut String, value: &str) {
    buf.push('"');
    buf.push_str(value);
    buf.push('"');
}

impl UglifyEmitVisitor {
    fn emit_null(&mut self) {
        self.buf.push_str(NULL);
    }
    fn emit_string(&mut self, s: &str) {
        push_quoted(&mut self.buf, s);
    }
    fn emit_number(&mut self, n: &str) {
        self.buf.push_str(n);
    }
    fn emit_boolean(&mut self, b: bool) {
        self.buf.push_str(if b { TRUE } else { FALSE });
    }
    fn emit_item_delim(&mut self) {
        self.buf.push(',');
    }
    fn emit_array_open(&mut self) {
        self.buf.push('[');
    }
    fn emit_array_close(&mut self) {
        self.buf.push(']');
    }
    fn emit_object_open(&mut self) {
        self.buf.push('{');
    }
    fn emit_object_close(&mut self) {
        self.buf.push('}');
    }
    fn emit_key_val_delim(&mut self) {
        self.buf.push(':');
    }
}

impl ValueVisitor<'_> for UglifyEmitVisitor {
    fn on_null(&mut self) {
        self.emit_null();
    }
    fn on_string(&mut self, s: &str) {
        self.emit_string(s);
    }
    fn on_number(&mut self, n: &str) {
        self.emit_number(n);
    }
    fn on_boolean(&mut self, b: bool) {
        self.emit_boolean(b);
    }
    fn on_item_delim(&mut self) {
        self.emit_item_delim();
    }
    fn on_array_open(&mut self) {
        self.emit_array_open();
    }
    fn on_array_close(&mut self) {
        self.emit_array_close();
    }
    fn on_object_open(&mut self) {
        self.emit_object_open();
    }
    fn on_object_close(&mut self) {
        self.emit_object_close();
    }
    fn on_object_key_val_delim(&mut self) {
        self.emit_key_val_delim();
    }
}

impl ParseVisitor<'_> for UglifyEmitVisitor {
    fn on_object_open(&mut self, _open_ctx: crate::tokens::TokenWithContext<'_>) {
        self.emit_object_open();
    }

    fn on_object_key(&mut self, key: &str) {
        self.emit_string(key);
    }

    fn on_object_key_val_delim(&mut self) {
        self.emit_key_val_delim();
    }

    fn on_object_close(&mut self, _range: std::ops::Range<usize>) {
        self.emit_object_close();
    }

    fn on_array_open(&mut self, _open_ctx: crate::tokens::TokenWithContext<'_>) {
        self.emit_array_open();
    }

    fn on_array_close(&mut self, _range: std::ops::Range<usize>) {
        self.emit_array_close();
    }

    fn on_scalar(&mut self, token_ctx: crate::tokens::TokenWithContext<'_>) {
        use crate::tokens::Token;
        match token_ctx.token {
            Token::Null => self.emit_null(),
            Token::Boolean(b) => self.emit_boolean(b),
            Token::String(s) => self.emit_string(s),
            Token::Number(n) => self.emit_number(n.as_ref()),
            _ => unreachable!(),
        }
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
