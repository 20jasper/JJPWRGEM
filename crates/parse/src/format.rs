use core::iter;

use crate::{
    Result,
    ast::{Value, parse_str},
    tokens::{FALSE, NULL, TRUE},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FormatOptions {
    key_val_delimiter: Option<(char, usize)>,
    indent: Option<(char, usize)>,
    eol: Option<(char, usize)>,
}

impl FormatOptions {
    pub fn new(
        key_val_delimiter: Option<(char, usize)>,
        indent: Option<(char, usize)>,
        eol: Option<(char, usize)>,
    ) -> Self {
        Self {
            key_val_delimiter,
            indent,
            eol,
        }
    }

    pub fn uglify() -> Self {
        Self {
            key_val_delimiter: None,
            indent: None,
            eol: None,
        }
    }

    pub fn prettify() -> Self {
        Self {
            key_val_delimiter: Some((' ', 1)),
            indent: Some((' ', 2)),
            eol: Some(('\n', 1)),
        }
    }
}

struct FormatBuf {
    opts: FormatOptions,
    buf: String,
    line_start: usize,
}

impl FormatBuf {
    fn new(buf: String, opts: FormatOptions) -> Self {
        Self {
            opts,
            buf,
            line_start: 0,
        }
    }

    fn push(&mut self, value: char) {
        self.buf.push(value);
    }
    fn push_str(&mut self, value: &str) {
        self.buf.push_str(value);
    }

    #[inline]
    fn push_quoted(&mut self, value: &str) {
        self.push('"');
        self.push_str(value);
        self.push('"');
    }

    #[inline]
    fn push_repeat(&mut self, c: char, count: usize) {
        self.buf.extend(iter::repeat_n(c, count));
    }

    #[inline]
    fn write_spec(&mut self, spec: Option<(char, usize)>) {
        if let Some((c, size)) = spec {
            self.push_repeat(c, size);
        }
    }

    pub fn write_key_val_delimiter(&mut self) {
        self.write_spec(self.opts.key_val_delimiter);
    }

    pub fn write_eol(&mut self) {
        self.write_spec(self.opts.eol);
        self.line_start = self.buf.len();
    }

    pub fn write_indent(&mut self, level: usize) {
        self.write_spec(self.opts.indent.map(|(c, size)| (c, size * level)));
    }

    fn into_inner(self) -> String {
        self.buf
    }
}

pub fn format_str<'a>(json: &'a str, options: FormatOptions) -> Result<'a, String> {
    let mut buf = FormatBuf::new(String::with_capacity(json.len()), options);
    format_value_into(&mut buf, &parse_str(json)?, 0);
    Ok(buf.into_inner())
}

/// writes formatted delimiters between formatted items
///
/// avoids allocating intermediate `String`s declaratively
/// # Examples
/// ```
/// # use jjpwrgem_parse::format::join_into;
/// # use std::fmt::Write as _;
///
/// let mut buf = String::new();
/// join_into(&mut buf, [1,2,3,4],
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

fn format_value_into(buf: &mut FormatBuf, val: &Value, depth: usize) {
    match val {
        Value::Null => buf.push_str(NULL),
        Value::String(s) => buf.push_quoted(s),
        Value::Number(s) => buf.push_str(s.as_ref()),
        Value::Object(entries) if entries.0.is_empty() => buf.push_str("{}"),
        Value::Object(entries) => {
            buf.push('{');
            buf.write_eol();
            join_into(
                buf,
                entries.0.iter(),
                |buf, (key, val)| {
                    buf.write_indent(depth + 1);
                    buf.push_quoted(key);
                    buf.push(':');
                    buf.write_key_val_delimiter();
                    format_value_into(buf, val, depth + 1);
                },
                |buf, _| {
                    buf.push(',');
                    buf.write_eol();
                },
            );
            buf.write_eol();
            buf.write_indent(depth);
            buf.push('}');
        }
        Value::Array(items) if items.is_empty() => buf.push_str("[]"),
        Value::Array(items) => {
            expanded_format_arr_into(buf, items, depth);
        }
        Value::Boolean(b) => buf.push_str(if *b { TRUE } else { FALSE }),
    }
}

fn expanded_format_arr_into(buf: &mut FormatBuf, items: &[Value], depth: usize) {
    buf.push('[');
    buf.write_eol();
    join_into(
        buf,
        items,
        |buf, val| {
            buf.write_indent(depth + 1);
            format_value_into(buf, val, depth + 1)
        },
        |buf, _| {
            buf.push(',');
            buf.write_eol();
        },
    );
    buf.write_eol();
    buf.write_indent(depth);
    buf.push(']');
}
pub fn format_value(val: &Value, options: &FormatOptions) -> String {
    let mut buf = FormatBuf::new(String::new(), *options);
    format_value_into(&mut buf, val, 0);
    buf.into_inner()
}

pub fn uglify_str(json: &str) -> Result<'_, String> {
    Ok(uglify_value(&parse_str(json)?))
}

pub fn uglify_value(val: &Value) -> String {
    format_value(val, &FormatOptions::uglify())
}

pub fn prettify_str(json: &str) -> Result<'_, String> {
    Ok(prettify_value(&parse_str(json)?))
}

pub fn prettify_value(val: &Value) -> String {
    format_value(val, &FormatOptions::prettify())
}
