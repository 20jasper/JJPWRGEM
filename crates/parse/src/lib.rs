#![feature(portable_simd, new_range)]
pub mod ast;
mod check;
pub mod diagnostics;
pub mod error;
pub mod format;
pub mod jsonlines;
mod node;
pub mod settings;
pub mod tokens;
mod traverse;

pub use check::validate_str;
pub use node::{Array, Items, Kind, Members, Node, Number, Object, Str};

pub use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxOptions {
    pub trailing_commas: bool,
    pub comments: bool,
}

impl SyntaxOptions {
    /// [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) compliant json: no trailing commas or comments
    pub const JSON: Self = Self {
        trailing_commas: false,
        comments: false,
    };

    /// [JSONC](https://jsonc.org/) json: trailing commas and comments allowed
    pub const JSONC: Self = Self {
        trailing_commas: true,
        comments: true,
    };
}

pub fn format_str(json: &str, request: &format::FormatRequest) -> Result<String> {
    match request {
        format::FormatRequest::Json(mode) => match mode {
            format::JsonMode::Prettify(opts) => {
                format::prettify_str(json, opts.preferred_width, opts.line_ending)
            }
            format::JsonMode::Uglify => format::uglify_str(json),
        },
        format::FormatRequest::Jsonlines(opts) => jsonlines::format(json, opts.line_ending),
    }
}
