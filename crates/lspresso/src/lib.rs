mod client;
mod core;
mod error;

pub use core::capabilities;

pub use client::{Client, Protocol};
pub use error::{Error, ErrorKind, Result};
pub use gen_lsp_types::{
    self, CodeActionResponse, Diagnostic, LanguageKind, Position, Range, TextEdit, Uri,
};

/// # Panics
///
/// Panics if `s` isn't a valid URI.
#[must_use]
pub fn parse_uri(s: &str) -> Uri {
    s.parse()
        .unwrap_or_else(|err| panic!("invalid URI {s:?}: {err}"))
}
