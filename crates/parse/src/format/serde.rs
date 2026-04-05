use crate::format;

mod error;
mod uglify;

pub use error::{Error, Result};

/// Serializes a [`serde::Serialize`] value using this crate's compact JSON
/// formatter.
///
/// ```
/// # use std::collections::HashMap;
/// # use jjpwrgem_parse::format::serde::uglify_serializable;
/// let map: HashMap<String, String> =
///     HashMap::from([("rust is a must".to_string(), "🦀".to_string())]);
/// assert_eq!(
///     uglify_serializable(map).unwrap(),
///     r#"{"rust is a must":"🦀"}"#
/// );
/// ```
pub fn uglify_serializable(val: impl serde::Serialize) -> Result<String> {
    let mut serializer = format::uglify::UglifyEmitVisitor::default();
    val.serialize(&mut serializer)?;
    Ok(serializer.buf)
}
