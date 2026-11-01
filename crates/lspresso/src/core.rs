#![cfg_attr(not(test), expect(dead_code, reason = "WIP and much is unused"))]
pub mod capabilities;
pub(crate) mod diagnostics;
mod docs;
pub(crate) mod inbox;
mod lifecycle;
mod server_requests;
pub(crate) mod text;

pub(crate) use lifecycle::ClientState;
pub(crate) use docs::DocumentTracker;

use crate::error::Error;

// Effects are kinda annoying but real real nice to squash concurrency bugs
// without a real server
#[derive(Debug)]
pub(crate) enum Effect {
    Resolved(lsp_server::RequestId, Result<serde_json::Value, Error>),
    Error(Error),
    Reply(lsp_server::Message),
}
