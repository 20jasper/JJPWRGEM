mod client;
mod convenience;
mod core;
mod error;
mod protocol;

pub use core::capabilities;

pub use client::Client;
pub use error::{Error, ErrorKind, Result};
pub use gen_lsp_types;
pub use protocol::Protocol;
