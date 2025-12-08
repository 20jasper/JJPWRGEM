use core::iter;

use crate::{
    Result,
    ast::{Value, parse_str},
    tokens::{FALSE, NULL, TRUE},
};

pub struct FormatOptions {
    key_val_delimiter: Option<(char, usize)>,
    array_value_delimiter: Option<(char, usize)>,
    indent: Option<(char, usize)>,
    eol: Option<(char, usize)>,
}

impl FormatOptions {
    pub fn new(
        key_val_delimiter: Option<(char, usize)>,
        array_value_delimiter: Option<(char, usize)>,
        indent: Option<(char, usize)>,
        eol: Option<(char, usize)>,
    ) -> Self {
        Self {
            key_val_delimiter,
            array_value_delimiter,
            indent,
            eol,
        }
    }

    pub fn uglify() -> Self {
        Self {
            key_val_delimiter: None,
            array_value_delimiter: None,
            indent: None,
            eol: None,
        }
    }

    pub fn prettify() -> Self {
        Self {
            key_val_delimiter: Some((' ', 1)),
            array_value_delimiter: Some((' ', 1)),
            indent: Some((' ', 4)),
            eol: Some(('\n', 1)),
        }
    }

    #[inline]
    fn push_repeat(buf: &mut String, c: char, count: usize) {
        buf.extend(iter::repeat_n(c, count));
    }

    #[inline]
    fn write_spec(buf: &mut String, spec: Option<(char, usize)>) {
        if let Some((c, size)) = spec {
            Self::push_repeat(buf, c, size);
        }
    }

    pub fn write_key_val_delimiter(&self, buf: &mut String) {
        Self::write_spec(buf, self.key_val_delimiter);
    }

    pub fn write_array_value_delimiter(&self, buf: &mut String) {
        Self::write_spec(buf, self.array_value_delimiter);
    }

    pub fn write_eol(&self, buf: &mut String) {
        Self::write_spec(buf, self.eol);
    }

    pub fn write_indent(&self, buf: &mut String, level: usize) {
        Self::write_spec(buf, self.indent.map(|(c, size)| (c, size * level)));
    }
}

pub fn format_str<'a>(json: &'a str, options: &FormatOptions) -> Result<'a, String> {
    let mut buf = String::with_capacity(json.len());
    format_value_into(&mut buf, &parse_str(json)?, options, 0);
    Ok(buf)
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
pub fn join_into<T>(
    buf: &mut String,
    items: impl IntoIterator<Item = T>,
    mut item_fmt: impl FnMut(&mut String, &T),
    mut delim_fmt: impl FnMut(&mut String, &T),
) {
    let mut iter = items.into_iter().peekable();
    while let Some(x) = iter.next() {
        item_fmt(buf, &x);
        if iter.peek().is_some() {
            delim_fmt(buf, &x);
        }
    }
}

pub fn format_value_into(buf: &mut String, val: &Value, options: &FormatOptions, depth: usize) {
    #[inline]
    fn push_quoted(buf: &mut String, value: &str) {
        buf.push('"');
        buf.push_str(value);
        buf.push('"');
    }
    match val {
        Value::Null => buf.push_str(NULL),
        Value::String(s) => push_quoted(buf, s),
        Value::Number(s) => buf.push_str(s.as_ref()),
        Value::Object(entries) => {
            buf.push('{');
            options.write_eol(buf);
            join_into(
                buf,
                entries.0.iter(),
                |buf, (key, val)| {
                    options.write_indent(buf, depth + 1);
                    push_quoted(buf, key);
                    buf.push(':');
                    options.write_key_val_delimiter(buf);
                    format_value_into(buf, val, options, depth + 1);
                },
                |buf, _| {
                    buf.push(',');
                    options.write_eol(buf);
                },
            );
            options.write_eol(buf);
            options.write_indent(buf, depth);
            buf.push('}');
        }
        Value::Array(items) if items.is_empty() => buf.push_str("[]"),
        Value::Array(items) => {
            buf.push('[');
            join_into(
                buf,
                items,
                |buf, val| format_value_into(buf, val, options, depth + 1),
                |buf, _| {
                    buf.push(',');
                    options.write_array_value_delimiter(buf);
                },
            );
            buf.push(']');
        }
        Value::Boolean(b) => buf.push_str(if *b { TRUE } else { FALSE }),
    }
}

pub fn format_value(val: &Value, options: &FormatOptions) -> String {
    let mut buf = String::new();
    format_value_into(&mut buf, val, options, 0);
    buf
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
