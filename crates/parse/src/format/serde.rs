// reexports to avoid versioning conflicts
pub use serde;
pub use serde_json;
pub use serde_json::{Error, Result};

use crate::{
    ast::Value,
    format::{self, FormatOptions, LineEnding},
};

impl From<serde_json::Value> for Value<'_> {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(b),
            serde_json::Value::Number(number) => Value::Number(number.to_string().into()),
            serde_json::Value::String(s) => {
                let s_static: &'static str = Box::leak(s.into_boxed_str());
                Value::String(s_static)
            }
            serde_json::Value::Array(values) => {
                Value::Array(values.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Object(map) => {
                let entries = map
                    .into_iter()
                    .map(|(k, v)| (&*k.leak(), v.into()))
                    .collect::<Vec<_>>()
                    .into();
                Value::Object(entries)
            }
        }
    }
}

/// A wrapper of [`crate::format::uglify_value`] that takes a
/// [`serde_json::Value`] instead of a [`crate::ast::Value`]
///
/// See [`uglify_value`] for a higher level API
/// ## Examples
/// ```
/// # use jjpwrgem_parse::format::serde::uglify_value;
/// let val = serde_json::json!({ "rust is a must": "🦀" });
/// assert_eq!(uglify_value(val), r#"{"rust is a must":"🦀"}"#);
/// ```
pub fn uglify_value(val: serde_json::Value) -> String {
    format::uglify_value(&val.into())
}

/// A wrapper of [`crate::format::uglify_value`] that takes a
/// [`serde::Serialize`]
///
/// See [`self::uglify_value`] for a lower level API
///
/// ```
/// # use std::collections::HashMap;
/// # use jjpwrgem_parse::format::serde::uglify_serializable;
/// let map: HashMap<String, String> = HashMap::from([
///     ("rust is a must".to_string(), "🦀".to_string()),
/// ]);
/// assert_eq!(uglify_serializable(map).unwrap(), r#"{"rust is a must":"🦀"}"#);
/// ```
pub fn uglify_serializable(val: impl serde::Serialize) -> serde_json::Result<String> {
    Ok(uglify_value(serde_json::to_value(val)?))
}

/// A wrapper of [`crate::format::prettify_value`] that takes a
/// [`serde_json::Value`] instead of a [`crate::ast::Value`]
///
/// See [`prettify_value`] for a higher level API
/// ## Examples
/// ```
/// # use jjpwrgem_parse::format::LineEnding;
/// # use jjpwrgem_parse::format::serde::prettify_value;
/// let val = serde_json::json!({ "rust is a must": "🦀" });
/// let expected = r#"{
///   "rust is a must": "🦀"
/// }"#;
/// assert_eq!(prettify_value(val, 80, LineEnding::Lf), expected);
/// ```
pub fn prettify_value(
    val: serde_json::Value,
    preferred_width: usize,
    line_ending: LineEnding,
) -> String {
    format::prettify_value(&val.into(), preferred_width, line_ending)
}

/// A wrapper of [`crate::format::prettify_value`] that takes a
/// [`serde::Serialize`]
///
/// See [`self::prettify_value`] for a lower level API
///
/// ```
/// # use std::collections::HashMap;
/// # use jjpwrgem_parse::format::LineEnding;
/// # use jjpwrgem_parse::format::serde::prettify_serializable;
/// let map: HashMap<String, String> = HashMap::from([
///     ("rust is a must".to_string(), "🦀".to_string()),
/// ]);
/// let expected = r#"{
///   "rust is a must": "🦀"
/// }"#;
/// assert_eq!(prettify_serializable(map, 80, LineEnding::Lf).unwrap(), expected);
/// ```
pub fn prettify_serializable(
    val: impl serde::Serialize,
    preferred_width: usize,
    line_ending: LineEnding,
) -> serde_json::Result<String> {
    Ok(prettify_value(
        serde_json::to_value(val)?,
        preferred_width,
        line_ending,
    ))
}

/// A wrapper of [`format::format_value`] that takes a
/// [`serde::Serialize`]
///
/// See [`self::format_value`] for a lower level API
///
/// ```
/// # use std::collections::HashMap;
/// # use jjpwrgem_parse::format::{FormatOptions, LineEnding};
/// # use jjpwrgem_parse::format::serde::format_serializable;
/// let map: HashMap<String, String> = HashMap::from([
///     ("rust is a must".to_string(), "🦀".to_string()),
/// ]);
/// let out = format_serializable(map, FormatOptions::prettify(LineEnding::Lf), 80).unwrap();
/// let expected = r#"{
///   "rust is a must": "🦀"
/// }"#;
/// assert_eq!(out, expected);
/// ```
pub fn format_serializable(
    val: impl serde::Serialize,
    options: FormatOptions,
    preferred_width: usize,
) -> serde_json::Result<String> {
    Ok(format_value(
        serde_json::to_value(val)?,
        options,
        preferred_width,
    ))
}

/// A wrapper of [`format::format_value`] that takes a
/// [`serde_json::Value`] instead of a [`crate::ast::Value`]
///
/// See [`format_value`] for a higher level API
/// ## Examples
/// ```
/// # use jjpwrgem_parse::format::{FormatOptions, LineEnding};
/// # use jjpwrgem_parse::format::serde::format_value;
/// let val = serde_json::json!({ "rust is a must": "🦀" });
/// let out = format_value(val, FormatOptions::prettify(LineEnding::Lf), 80);
/// let expected = r#"{
///   "rust is a must": "🦀"
/// }"#;
/// assert_eq!(out, expected);
/// ```
pub fn format_value(
    val: serde_json::Value,
    options: FormatOptions,
    preferred_width: usize,
) -> String {
    format::format_value(&val.into(), &options, preferred_width)
}
