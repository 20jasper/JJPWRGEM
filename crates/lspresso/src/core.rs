pub mod capabilities;
pub(crate) mod diagnostics;
mod docs;
pub(crate) mod inbox;
mod lifecycle;
pub(crate) mod text;
mod server_requests;

pub(crate) use docs::DocumentTracker;
pub(crate) use inbox::{Inbox, Verdict};
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
