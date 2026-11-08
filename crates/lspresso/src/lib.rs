//! # lspresso
//!
//! lsp testing and benchmarking
//!
//! ![an orange axolotl in a blue coffee cup falling over. coffee is spilling out and the axolotl looks distressed](./lspresso.webp)
//!
//! ## limitations
//!
//! - server to client requests get a canned response. They could be scriptable and assertable

mod client;
mod command;
mod convenience;
mod core;
mod error;
mod protocol;
#[cfg(feature = "test-binaries")]
#[doc(hidden)]
pub mod scripted_server;

pub use core::capabilities;

pub use client::Client;
pub use command::server_command;
pub use error::{Error, ErrorKind, Result};
pub use gen_lsp_types;
pub use protocol::Protocol;
