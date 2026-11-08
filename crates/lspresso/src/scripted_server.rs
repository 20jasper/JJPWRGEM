//! What a test tells the scripted server to do, passed as JSON in `LSPRESSO_CONFIG`.

use std::path::PathBuf;

use gen_lsp_types::ServerCapabilities;
use lsp_server::{Message, ResponseError};
use serde::{Deserialize, Serialize};

pub const ENV: &str = "LSPRESSO_CONFIG";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// One step per request, in order. Past the end, requests are ignored.
    pub steps: Vec<Step>,
    /// Answer requests past the end of `steps` instead of ignoring them.
    pub respond_to_all: bool,
    /// Capabilities in the initialize result.
    pub capabilities: ServerCapabilities,
    pub publications: Publications,
    /// Append each received message to this file as a line of JSON.
    pub record: Option<PathBuf>,
}

/// What a document sync is answered with.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Publications {
    /// A stale publication, then one for the version sent.
    #[default]
    Versioned,
    /// One publication with no version.
    Unversioned,
    Off,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    Respond {
        delay_ms: u64,
        outcome: Outcome,
    },
    /// Writes a message, then handles the same request with the next step.
    Send(Message),
    /// Writes the text as is, then handles the same request with the next step.
    WriteRaw(String),
    Ignore,
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Outcome {
    /// The initialize result, or `null` for any other request.
    Canned,
    Result(serde_json::Value),
    Error(ResponseError),
}
