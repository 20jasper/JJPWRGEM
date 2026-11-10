use std::path::PathBuf;

use gen_lsp_types::*;
use lsp_server::{Message, ResponseError};
use serde::{Deserialize, Serialize};

pub const ENV: &str = "LSPRESSO_CONFIG";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub steps: Vec<Step>,
    pub respond_to_all: bool,
    pub capabilities: ServerCapabilities,
    pub publications: Publications,
    pub record: Option<PathBuf>,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Publications {
    #[default]
    Versioned,
    Unversioned,
    Off,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Step {
    Respond { delay_ms: u64, outcome: Outcome },
    Send(Message),
    WriteRaw(String),
    Ignore,
    Exit,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Outcome {
    Canned,
    Result(serde_json::Value),
    Error(ResponseError),
}

pub fn notification<N: Notification>(params: N::Params) -> Message {
    Message::Notification(crate::core::notification_buffer::notification::<N>(params))
}

pub fn parse<N: Notification>(notification: &lsp_server::Notification) -> Option<N::Params> {
    crate::core::notification_buffer::parse::<N>(notification)
}
