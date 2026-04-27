mod prettify;
#[cfg(feature = "serde")]
pub mod serde;
mod uglify;

pub use prettify::{
    FormatOptions, format_str, format_value, prettify_str, prettify_value, prettify_value_into,
};
pub use uglify::{uglify_str, uglify_value, uglify_value_into};

use crate::tokens::{FALSE, NULL, TRUE};

/// writes formatted delimiters between formatted items
///
/// avoids allocating intermediate `String`s declaratively
/// # Examples
/// ```
/// # use jjpwrgem_parse::format::join_into;
/// # use std::fmt::Write as _;
///
/// let mut buf = String::new();
/// join_into(
///     &mut buf,
///     [1, 2, 3, 4],
///     |buf, x| write!(buf, "{}", x * 2).unwrap(),
///     |buf, _| write!(buf, ",").unwrap(),
/// );
/// assert_eq!(buf, "2,4,6,8");
/// ```
pub fn join_into<T, B>(
    buf: &mut B,
    items: impl IntoIterator<Item = T>,
    mut item_fmt: impl FnMut(&mut B, &T),
    mut delim_fmt: impl FnMut(&mut B, &T),
) {
    let mut iter = items.into_iter();
    if let Some(first) = iter.next() {
        item_fmt(buf, &first);
        for item in iter {
            delim_fmt(buf, &item);
            item_fmt(buf, &item);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl LineEnding {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}

pub trait Emitter {
    fn push(&mut self, c: char);
    fn push_str(&mut self, s: &str);

    // defaults
    fn push_quoted(&mut self, s: &str) {
        self.push('"');
        self.push_str(s);
        self.push('"');
    }

    fn emit_null(&mut self) {
        self.push_str(NULL);
    }
    fn emit_string(&mut self, s: &str) {
        self.push_quoted(s);
    }
    fn emit_mantissa(&mut self, n: &str) {
        self.push_str(n);
    }
    fn emit_exponent(&mut self, e: &str) {
        self.push('e');
        self.push_str(e);
    }
    fn emit_boolean(&mut self, b: bool) {
        self.push_str(if b { TRUE } else { FALSE });
    }
    fn emit_item_delim(&mut self) {
        self.push(',');
    }
    fn emit_array_open(&mut self) {
        self.push('[');
    }
    fn emit_array_close(&mut self) {
        self.push(']');
    }
    fn emit_object_open(&mut self) {
        self.push('{');
    }
    fn emit_object_close(&mut self) {
        self.push('}');
    }
    fn emit_key_val_delim(&mut self) {
        self.push(':');
    }
}
