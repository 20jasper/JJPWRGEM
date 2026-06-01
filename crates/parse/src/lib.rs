#![feature(portable_simd, new_range)]
pub mod ast;
mod check;
pub mod diagnostics;
pub mod error;
pub mod format;
pub mod jsonlines;
pub mod tokens;
mod traverse;

pub use check::validate_str;

pub use crate::error::{Error, ErrorKind, Result};

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
