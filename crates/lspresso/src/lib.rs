//! # lspresso
//!
//! language server test client
//!
//! ![an orange axolotl in a blue coffee cup falling over. coffee is spilling out and the axolotl looks distressed](./lspresso.webp)

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
