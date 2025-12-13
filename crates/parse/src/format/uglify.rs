use crate::{
    Result,
    ast::{Value, parse_str},
    format::join_into,
    tokens::{FALSE, NULL, TRUE},
};

pub fn uglify_str(json: &str) -> Result<'_, String> {
    Ok(uglify_value(&parse_str(json)?))
}

pub fn uglify_value(val: &Value) -> String {
    let mut buf = String::new();
    uglify_value_into(&mut buf, val);
    buf
}

fn uglify_value_into(buf: &mut String, val: &Value) {
    fn push_quoted(buf: &mut String, value: &str) {
        buf.push('"');
        buf.push_str(value);
        buf.push('"');
    }
    match val {
        Value::Null => buf.push_str(NULL),
        Value::String(s) => push_quoted(buf, s),
        Value::Number(s) => buf.push_str(s.as_ref()),
        Value::Object(entries) if entries.0.is_empty() => buf.push_str("{}"),
        Value::Object(entries) => {
            buf.push('{');
            join_into(
                buf,
                &entries.0,
                |buf, (key, value)| {
                    push_quoted(buf, key);
                    buf.push(':');
                    uglify_value_into(buf, value);
                },
                |buf, _| {
                    buf.push(',');
                },
            );
            buf.push('}');
        }
        Value::Array(items) if items.is_empty() => buf.push_str("[]"),
        Value::Array(items) => {
            buf.push('[');
            join_into(
                buf,
                items.iter(),
                |buf, value| uglify_value_into(buf, value),
                |buf, _| {
                    buf.push(',');
                },
            );
            buf.push(']');
        }
        Value::Boolean(b) => buf.push_str(if *b { TRUE } else { FALSE }),
    }
}
