use std::{io, time::Instant};

use displaydoc::Display;
use gen_lsp_types::{Range, Uri};
use lsp_server::RequestId;

/// {0}
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(Box<ErrorKind>);

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self(Box::new(value))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        ErrorKind::Io(err).into()
    }
}

impl From<Error> for ErrorKind {
    fn from(err: Error) -> Self {
        *err.0
    }
}

impl std::ops::Deref for Error {
    type Target = ErrorKind;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

#[derive(Display, thiserror::Error, Debug)]
pub enum ErrorKind {
    /// server io: {0}
    Io(#[from] io::Error),
    /// failed to spawn {program}: {source}
    Spawn {
        program: String,
        #[source]
        source: io::Error,
    },
    /// server disconnected while waiting for response to {method}
    Disconnected { method: &'static str },
    /// server sent a malformed message while waiting for response to {method}: {source}
    MalformedMessage {
        method: &'static str,
        #[source]
        source: io::Error,
    },
    /// {method} (id={id}) responded with error {code}: {message}
    Response {
        code: i32,
        message: String,
        method: &'static str,
        id: RequestId,
        data: Option<serde_json::Value>,
    },
    /// timed out after {after_ms}ms waiting for response to {method} (id={id})
    ResponseTimeout {
        method: &'static str,
        id: RequestId,
        after_ms: u64,
    },
    /// client already shut down: spawn a new Client for another session
    AlreadyShutdown,
    /// timed out after {after_ms}ms waiting for the server process to exit
    ExitTimeout { after_ms: u64 },
    /// not yet initialized: call initialize before sending protocol requests
    NotInitialized,
    /// already initialized: call initialize only once per session
    AlreadyInitialized,
    /// response id {id} doesn't match any request lspresso sent
    UnmatchedResponseId { id: RequestId },
    /// {uri} is already open: call did_close before opening it again
    AlreadyOpen { uri: Uri },
    /// {uri} is not open: call did_open before sending document notifications for it
    NotOpen { uri: Uri },
    /// edit {index} has a range that isn't valid in the document: {range:?}
    InvalidEdit { index: usize, range: Range },
    /// edits {a} and {b} overlap
    OverlappingEdits { a: usize, b: usize },
    /// exhausted available document versions for {uri}
    DocumentVersionExhausted { uri: Uri },
    /// {method} response was malformed: {message}
    MalformedResponse {
        method: &'static str,
        message: String,
    },
    /// timed out after {after_ms}ms waiting for diagnostics for {uri}
    DiagnosticsTimeout { uri: Uri, after_ms: u64 },
    /// timed out after {after_ms}ms waiting for a matching {method} notification
    NotificationTimeout { method: &'static str, after_ms: u64 },
}

pub(crate) fn elapsed_ms(start: Instant, now: Instant) -> u64 {
    now.duration_since(start)
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub type Result<T> = std::result::Result<T, Error>;
