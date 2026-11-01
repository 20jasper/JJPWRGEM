mod client;
mod core;
mod error;

pub use core::capabilities;

pub use client::Client;
pub use error::{Error, ErrorKind, Result};
pub use gen_lsp_types;
