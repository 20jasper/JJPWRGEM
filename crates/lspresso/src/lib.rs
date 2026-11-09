//! # lspresso
//!
//! Use existing test helpers and benchmarking tools to validate and performance and correctness!
//!
//! ![an orange axolotl in a blue coffee cup falling over. coffee is spilling out and the axolotl looks distressed](./lspresso.webp)
//!
//! ```no_run
//! # use std::{process::Command, time::Duration};
//! use lspresso::{Client, capabilities, gen_lsp_types::*};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut client = Client::spawn_initialized(
//!     Command::new("my-lsp"),
//!     // use capabilities captured from real editors
//!     // emacs, helix, neovim, and vscode are supported
//!     capabilities::vscode::latest(),
//! )?;
//! let uri: Uri = "hi.json".parse()?;
//!
//! // convenience methods provide a simpler API
//! client.did_open(uri.clone(), "{}", LanguageKind::JSON)?;
//!
//! // for example, `Client::diagnostics` pulls when the server supports it and
//! // waits for pushed diagnostics otherwise
//! let diags = client.diagnostics(&uri)?;
//! assert_eq!(diags, vec![]);
//!
//! // if you need lower level control, import the `Protocol` trait
//! // each method is prefixed with `protocol_` to differentiate from the
//! // convenience layer
//! use lspresso::Protocol as _;
//!
//! // pull diagnostics
//! let diags = client.protocol_diagnostic(DocumentDiagnosticParams {
//!     text_document: TextDocumentIdentifier { uri: uri.clone() },
//!     identifier: Some("my-lsp".into()),
//!     previous_result_id: Some("diagnostics-1".into()),
//!     work_done_progress_params: WorkDoneProgressParams {
//!         work_done_token: Some(ProgressToken::String("work-done-1".into())),
//!     },
//!     partial_result_params: PartialResultParams {
//!         partial_result_token: Some(ProgressToken::String("partial-1".into())),
//!     },
//! })?;
//!
//! // push diagnostics wait for a notification or the timeout
//! let published = client.wait_for_notification_where::<PublishDiagnosticsNotification>(
//!     Duration::from_secs(5),
//!     |p| p.uri == uri,
//! )?;
//! assert!(published.diagnostics.is_empty());
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Benchmarking
//!
//! Use existing benchmarking tools like criterion to benchmark
//!
//! ```no_run
//! # use std::process::Command;
//! # use criterion::{Criterion, criterion_group, criterion_main};
//! # use lspresso::{Client, capabilities, gen_lsp_types::*};
//! fn bench_diagnostics(c: &mut Criterion) {
//!     let mut client =
//!         Client::spawn_initialized(Command::new("my-lsp"), capabilities::vscode::latest())
//!             .unwrap();
//!     let uri: Uri = "bench.json".parse().unwrap();
//!     client
//!         .did_open(uri.clone(), r#"{"a": [1, 2, 3]}"#, LanguageKind::JSON)
//!         .unwrap();
//!
//!     c.bench_function("diagnostics", |b| {
//!         b.iter(|| client.diagnostics(&uri).unwrap());
//!     });
//!
//!     // peak RSS over the session
//!     let rss = client.rss_bytes().expect("the server is running");
//!     println!("server RSS: {} MiB", rss / (1024 * 1024));
//! }
//!
//! criterion_group!(benches, bench_diagnostics);
//! criterion_main!(benches);
//! ```
//!
//! ## limitations
//!
//! - server to client requests get a canned response. They could be scriptable and assertable
//! - stdout is supported but not node ipc or tcp

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
