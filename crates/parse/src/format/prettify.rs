use core::iter;

use crate::{
    Result,
    ast::{Value, parse_str},
    format::LineEnding,
    tokens::{FALSE, NULL, TRUE},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FormatOptions {
    key_val_delimiter: Option<(char, usize)>,
    indent: Option<(char, usize)>,
    line_ending: LineEnding,
}

impl FormatOptions {
    pub fn new(
        key_val_delimiter: Option<(char, usize)>,
        indent: Option<(char, usize)>,
        line_ending: LineEnding,
    ) -> Self {
        Self {
            key_val_delimiter,
            indent,
            line_ending,
        }
    }

    pub fn prettify(line_ending: LineEnding) -> Self {
        Self {
            key_val_delimiter: Some((' ', 1)),
            indent: Some((' ', 2)),
            line_ending,
        }
    }
}

struct FormatBuf {
    opts: FormatOptions,
    buf: String,
    line_start: usize,
    preferred_width: usize,
}

impl FormatBuf {
    fn new(buf: String, opts: FormatOptions, preferred_width: usize) -> Self {
        Self {
            opts,
            buf,
            line_start: 0,
            preferred_width,
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
        self.push_str(self.opts.line_ending.as_str());
        self.line_start = self.buf.len();
    }

    pub fn write_indent(&mut self, level: usize) {
        self.write_spec(self.opts.indent.map(|(c, size)| (c, size * level)));
    }

    fn into_inner(self) -> String {
        self.buf
    }

    pub fn column(&self) -> usize {
        self.buf.len() - self.line_start
    }

    fn available_bytes(&self) -> usize {
        self.preferred_width.saturating_sub(self.column())
    }
}

pub fn format_str<'a>(
    json: &'a str,
    options: FormatOptions,
    preferred_width: usize,
) -> Result<'a, String> {
    let mut buf = FormatBuf::new(String::with_capacity(json.len()), options, preferred_width);
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
            if len::should_expand(val, buf.available_bytes()) {
                expanded_format_arr_into(buf, items, depth)
            } else {
                compact_format_arr_into(buf, items, depth);
            }
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

fn compact_format_arr_into(buf: &mut FormatBuf, items: &[Value], depth: usize) {
    buf.push('[');
    join_into(
        buf,
        items,
        |buf, val| format_value_into(buf, val, depth + 1),
        |buf, _| {
            buf.push(',');
            buf.write_key_val_delimiter();
        },
    );
    buf.push(']');
}

pub fn format_value(val: &Value, options: &FormatOptions, preferred_width: usize) -> String {
    let mut buf = FormatBuf::new(String::new(), *options, preferred_width);
    format_value_into(&mut buf, val, 0);
    buf.into_inner()
}

pub fn prettify_str(
    json: &str,
    preferred_width: usize,
    line_ending: LineEnding,
) -> Result<'_, String> {
    format_str(json, FormatOptions::prettify(line_ending), preferred_width)
}

pub fn prettify_value(val: &Value, preferred_width: usize, line_ending: LineEnding) -> String {
    format_value(val, &FormatOptions::prettify(line_ending), preferred_width)
}

mod len {
    use crate::{
        ast::Value,
        tokens::{FALSE, NULL, TRUE},
    };

    /// returns if the inline length of the value > limit or it finds a newline
    pub fn should_expand(val: &Value, limit: usize) -> bool {
        try_get_value_len(val, limit).is_none()
    }

    fn try_get_value_len(val: &Value<'_>, limit: usize) -> Option<usize> {
        fn within_limit(len: usize, limit: usize) -> bool {
            len <= limit
        }

        let len = match val {
            Value::Null => NULL.len(),
            Value::String(s) => s.len(),
            Value::Number(s) => s.len(),
            Value::Object(entries) => {
                if entries.is_empty() {
                    2
                } else {
                    return None; // will have a newline
                }
            }
            Value::Array(values) => {
                let brackets_len = 2;
                let mut sum = brackets_len;
                let mut values = values.iter();
                while let Some(value) = values.next()
                    && within_limit(sum, limit)
                {
                    let remaining = limit.saturating_sub(sum);
                    let len = try_get_value_len(value, remaining)?;
                    sum += len;
                }

                sum
            }
            Value::Boolean(b) => {
                if *b {
                    TRUE.len()
                } else {
                    FALSE.len()
                }
            }
        };

        within_limit(len, limit).then_some(len)
    }
}
