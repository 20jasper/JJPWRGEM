use core::range::Range;

use displaydoc::Display;

use crate::{error::Error::FalseRoot, primitive::PrimitiveType};

#[derive(Debug, Display, PartialEq, Eq, Clone)]
pub enum Error {
    /// mismatched types. expected {expected} but found {actual}
    MismatchedTypes {
        range: Range<usize>,
        expected: PrimitiveType,
        actual: PrimitiveType,
    },
    /// schema root must be a boolean or object. found {found}
    InvalidSchemaRoot {
        range: Range<usize>,
        found: PrimitiveType,
    },
    /// a literal `false` always fails to validate
    FalseRoot { range: Range<usize> },
    /// invalid type {found}
    InvalidType { range: Range<usize>, found: String },
    /// `properties` must be an object. found {found}
    InvalidProperties {
        range: Range<usize>,
        found: PrimitiveType,
    },
    /// `items` must be a schema or an array of schemas. found {found}
    InvalidItems {
        range: Range<usize>,
        found: PrimitiveType,
    },
}

impl Error {
    pub fn range(&self) -> &Range<usize> {
        match self {
            Error::MismatchedTypes { range, .. }
            | Error::InvalidSchemaRoot { range, .. }
            | Error::InvalidType { range, .. }
            | Error::InvalidProperties { range, .. }
            | Error::InvalidItems { range, .. }
            | FalseRoot { range } => range,
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
