#![feature(portable_simd, new_range)]
pub mod ast;
mod check;
pub mod diagnostics;
pub mod error;
pub mod format;
pub mod tokens;
mod traverse;

pub use check::validate_str;

pub use crate::error::{Error, ErrorKind, Result};
