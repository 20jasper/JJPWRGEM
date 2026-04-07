#![feature(portable_simd)]
pub mod ast;
mod check;
pub mod error;
pub mod format;
pub mod tokens;
mod traverse;

pub use check::validate_str;

pub use crate::error::{Error, ErrorKind, Result};
