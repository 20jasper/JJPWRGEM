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

    fn get(opts: Option<(char, usize)>) -> String {
        if let Some((c, size)) = opts {
            [c].repeat(size).into_iter().collect()
        } else {
            "".into()
        }
    }

    pub fn get_key_val_delimiter(&self) -> String {
        Self::get(self.key_val_delimiter)
    }

    pub fn get_array_value_delimiter(&self) -> String {
        Self::get(self.array_value_delimiter)
    }

    pub fn get_eol(&self) -> String {
        Self::get(self.eol)
    }

    pub fn get_indent(&self, level: usize) -> String {
        Self::get(self.indent.map(|(c, size)| (c, size * level)))
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
            let kv_delim = options.get_key_val_delimiter();
            let key_indent = options.get_indent(depth + 1);
            let eol = options.get_eol();
            let closing_indent = options.get_indent(depth);

            buf.push('{');
            buf.push_str(&eol);
            join_into(
                buf,
                entries.0.iter(),
                |buf, (key, val)| {
                    buf.push_str(&key_indent);
                    push_quoted(buf, key);
                    buf.push(':');
                    buf.push_str(&kv_delim);
                    format_value_into(buf, val, options, depth + 1);
                },
                |buf, _| {
                    buf.push(',');
                    buf.push_str(&eol);
                },
            );
            buf.push_str(&eol);
            buf.push_str(&closing_indent);
            buf.push('}');
        }
        Value::Array(items) if items.is_empty() => buf.push_str("[]"),
        Value::Array(items) => {
            let delimiter = options.get_array_value_delimiter();
            buf.push('[');
            join_into(
                buf,
                items,
                |buf, val| format_value_into(buf, val, options, depth + 1),
                |buf, _| {
                    buf.push(',');
                    buf.push_str(&delimiter);
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
