use core::{iter, range::Range};

use crate::{
    Result,
    ast::{Document, Value},
    format::LineEnding,
    tokens::{FALSE, NULL, TRUE},
};

const QUOTE_LEN: usize = 1;
const QUOTE_PAIR_LEN: usize = QUOTE_LEN * 2;
const COMMA_LEN: usize = 1;
const COLON_LEN: usize = 1;
const EXPONENT_MARKER_LEN: usize = 1;
const BRACKET_PAIR_LEN: usize = 2;
const BRACE_PAIR_LEN: usize = 2;

type ObjectEntry = (Range<usize>, Value);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct FormatOptions {
    key_val_delimiter: Option<(char, usize)>,
    indent: Option<(char, usize)>,
    line_ending: LineEnding,
}

impl FormatOptions {
    pub(crate) fn prettify(line_ending: LineEnding) -> Self {
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

    fn delimiter_len(&self) -> usize {
        self.opts.key_val_delimiter.map_or(0, |(_, size)| size)
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

pub(crate) fn format_str(
    json: &str,
    options: FormatOptions,
    preferred_width: usize,
) -> Result<String> {
    let mut buf = FormatBuf::new(String::with_capacity(json.len()), options, preferred_width);
    let doc = Document::parse(json)?;
    format_document_value_into(&mut buf, &doc, doc.root(), 0);
    Ok(buf.into_inner())
}

use super::join_into;

fn range_len(range: Range<usize>) -> usize {
    range.end - range.start
}

fn number_len(mantissa: Range<usize>, exponent: Option<Range<usize>>) -> usize {
    range_len(mantissa)
        + if let Some(exponent) = exponent {
            EXPONENT_MARKER_LEN + range_len(exponent)
        } else {
            0
        }
}

fn format_document_value_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    val: &Value,
    depth: usize,
) {
    match val {
        Value::Null => buf.push_str(NULL),
        Value::String(r) => buf.push_quoted(doc.slice(*r)),
        Value::Number { mantissa, exponent } => {
            buf.push_str(doc.slice(*mantissa));
            if let Some(exponent) = exponent {
                buf.push('e');
                buf.push_str(doc.slice(*exponent));
            }
        }
        Value::Object(entries) if entries.0.is_empty() => buf.push_str("{}"),
        Value::Object(entries) => {
            if !len::should_expand(doc, val, buf.available_bytes(), buf.delimiter_len()) {
                compact_format_obj_into(buf, doc, entries.0.as_slice(), depth);
            } else {
                expanded_format_obj_into(buf, doc, entries.0.as_slice(), depth);
            }
        }
        Value::Array(items) if items.is_empty() => buf.push_str("[]"),
        Value::Array(items) => {
            if !len::should_expand(doc, val, buf.available_bytes(), buf.delimiter_len()) {
                compact_format_arr_into(buf, doc, items, depth);
            } else if items.iter().all(|v| matches!(v, Value::Number { .. })) {
                fill_format_arr_into(buf, doc, items, depth);
            } else {
                expanded_format_arr_into(buf, doc, items, depth);
            }
        }
        Value::Boolean(b) => buf.push_str(if *b { TRUE } else { FALSE }),
    }
}

fn expanded_format_arr_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    items: &[Value],
    depth: usize,
) {
    buf.push('[');
    buf.write_eol();
    join_into(
        buf,
        items,
        |buf, val| {
            buf.write_indent(depth + 1);
            format_document_value_into(buf, doc, val, depth + 1);
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

fn expanded_format_obj_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    entries: &[ObjectEntry],
    depth: usize,
) {
    buf.push('{');
    buf.write_eol();
    join_into(
        buf,
        entries.iter(),
        |buf, (key_range, val)| {
            buf.write_indent(depth + 1);
            write_object_entry_into(buf, doc, key_range, val, depth + 1);
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

fn fill_format_arr_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    items: &[Value],
    depth: usize,
) {
    buf.push('[');
    buf.write_eol();
    buf.write_indent(depth + 1);
    for (i, item) in items.iter().enumerate() {
        let Value::Number { mantissa, exponent } = item else {
            unreachable!("fill_format_arr_into called with non-Number item");
        };
        let item_len = number_len(*mantissa, *exponent);
        if i > 0 {
            buf.push(',');
            let trailing_comma_len = usize::from(i + 1 < items.len());
            if item_len + COMMA_LEN + trailing_comma_len > buf.available_bytes() {
                buf.write_eol();
                buf.write_indent(depth + 1);
            } else {
                buf.write_key_val_delimiter();
            }
        }
        format_document_value_into(buf, doc, item, depth + 1);
    }
    buf.write_eol();
    buf.write_indent(depth);
    buf.push(']');
}

fn compact_format_arr_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    items: &[Value],
    depth: usize,
) {
    buf.push('[');
    join_into(
        buf,
        items,
        |buf, val| format_document_value_into(buf, doc, val, depth + 1),
        |buf, _| {
            buf.push(',');
            buf.write_key_val_delimiter();
        },
    );
    buf.push(']');
}

fn write_object_entry_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    key_range: &Range<usize>,
    val: &Value,
    depth: usize,
) {
    buf.push_quoted(doc.slice(*key_range));
    buf.push(':');
    buf.write_key_val_delimiter();
    format_document_value_into(buf, doc, val, depth);
}

fn compact_format_obj_into<S: AsRef<str>>(
    buf: &mut FormatBuf,
    doc: &Document<S>,
    entries: &[ObjectEntry],
    depth: usize,
) {
    buf.push('{');
    buf.write_key_val_delimiter();
    join_into(
        buf,
        entries.iter(),
        |buf, (key_range, val)| write_object_entry_into(buf, doc, key_range, val, depth + 1),
        |buf, _| {
            buf.push(',');
            buf.write_key_val_delimiter();
        },
    );
    buf.write_key_val_delimiter();
    buf.push('}');
}

fn format_document<S: AsRef<str>>(
    doc: &Document<S>,
    options: &FormatOptions,
    preferred_width: usize,
) -> String {
    let mut buf = FormatBuf::new(String::new(), *options, preferred_width);
    format_document_value_into(&mut buf, doc, doc.root(), 0);
    buf.into_inner()
}

pub fn prettify_str(json: &str, preferred_width: usize, line_ending: LineEnding) -> Result<String> {
    format_str(json, FormatOptions::prettify(line_ending), preferred_width)
}

pub fn prettify_document<S: AsRef<str>>(
    doc: &Document<S>,
    preferred_width: usize,
    line_ending: LineEnding,
) -> String {
    format_document(doc, &FormatOptions::prettify(line_ending), preferred_width)
}

pub fn prettify_document_into<S: AsRef<str>>(
    buf: &mut String,
    doc: &Document<S>,
    preferred_width: usize,
    line_ending: LineEnding,
) {
    let mut fmt_buf = FormatBuf::new(
        std::mem::take(buf),
        FormatOptions::prettify(line_ending),
        preferred_width,
    );
    format_document_value_into(&mut fmt_buf, doc, doc.root(), 0);
    *buf = fmt_buf.into_inner();
}

mod len {
    use super::{
        BRACE_PAIR_LEN, BRACKET_PAIR_LEN, COLON_LEN, COMMA_LEN, QUOTE_PAIR_LEN, number_len,
    };
    use crate::{
        ast::Value,
        tokens::{FALSE, NULL, TRUE},
    };

    /// returns if the inline length of the value > limit or it finds a newline
    pub fn should_expand<S: AsRef<str>>(
        doc: &crate::ast::Document<S>,
        val: &Value,
        limit: usize,
        delimiter_len: usize,
    ) -> bool {
        try_get_value_len(doc, val, limit, delimiter_len).is_none()
    }

    fn quoted_len(value: &str) -> usize {
        value.chars().count() + QUOTE_PAIR_LEN
    }

    fn separated_item_prefix_len(delimiter_len: usize) -> usize {
        COMMA_LEN + delimiter_len
    }

    fn try_get_value_len<S: AsRef<str>>(
        doc: &crate::ast::Document<S>,
        val: &Value,
        limit: usize,
        delimiter_len: usize,
    ) -> Option<usize> {
        fn within_limit(len: usize, limit: usize) -> bool {
            len <= limit
        }

        let len = match val {
            Value::Null => NULL.len(),
            Value::String(r) => quoted_len(doc.slice(*r)),
            Value::Number { mantissa, exponent } => number_len(*mantissa, *exponent),
            Value::Object(entries) => {
                if entries.is_empty() {
                    BRACE_PAIR_LEN
                } else {
                    let braces_len = BRACE_PAIR_LEN + (delimiter_len * 2);
                    let mut sum = braces_len;
                    for (i, (key_range, value)) in entries.0.iter().enumerate() {
                        if !within_limit(sum, limit) {
                            break;
                        }
                        if i > 0 {
                            sum += separated_item_prefix_len(delimiter_len);
                        }
                        sum += quoted_len(doc.slice(*key_range));
                        sum += COLON_LEN + delimiter_len;
                        let remaining = limit.saturating_sub(sum);
                        let len = try_get_value_len(doc, value, remaining, delimiter_len)?;
                        sum += len;
                    }
                    sum
                }
            }
            Value::Array(values) => {
                let brackets_len = BRACKET_PAIR_LEN;
                let mut sum = brackets_len;
                for (i, value) in values.iter().enumerate() {
                    if !within_limit(sum, limit) {
                        break;
                    }
                    if i > 0 {
                        sum += separated_item_prefix_len(delimiter_len);
                    }
                    let remaining = limit.saturating_sub(sum);
                    let len = try_get_value_len(doc, value, remaining, delimiter_len)?;
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
