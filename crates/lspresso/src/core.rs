pub mod capabilities;
pub(crate) mod diagnostics;
mod documents;
mod lifecycle;
pub(crate) mod notification_buffer;
mod server_requests;
pub(crate) mod text;

pub(crate) use documents::DocumentTracker;
pub(crate) use lifecycle::ClientState;

use crate::error::Error;

// Effects are kinda annoying but real real nice to squash concurrency bugs
// without a real server
#[derive(Debug)]
pub(crate) enum Effect {
    Resolved(lsp_server::RequestId, Result<serde_json::Value, Error>),
    Error(Error),
    Reply(lsp_server::Message),
}
