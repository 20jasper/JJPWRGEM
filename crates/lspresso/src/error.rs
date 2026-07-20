use std::{fmt, io, time::Instant};

use displaydoc::Display;
use gen_lsp_types::Uri;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExitCode(pub(crate) Option<i32>);

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.0 {
            write!(f, " (exit code: {code})")
        } else {
            Ok(())
        }
    }
}

/// {0}
#[derive(Display, thiserror::Error)]
pub struct Error(Box<ErrorKind>);

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self(Box::new(value))
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
    /// failed to serialize params for {method}: {source}
    SerializeError {
        source: serde_json::Error,
        method: String,
    },
    /// failed to deserialize result for {method}: {source}
    DeserializeError {
        source: serde_json::Error,
        method: String,
        raw: serde_json::Value,
    },
    /// {method} (id={id}) responded with error {code}: {message}
    Response {
        code: i32,
        message: String,
        method: String,
        id: i32,
        data: Option<serde_json::Value>,
    },
    /// timed out after {after_ms}ms waiting for response to {method} (id={id}); set RUST_LOG=lspresso=debug for a message trace
    ResponseTimeout {
        method: String,
        id: i32,
        after_ms: u64,
    },
    /// server closed the connection{exit_code} while waiting for response to {method} (id={id})
    ResponseDisconnected {
        method: String,
        id: i32,
        exit_code: ExitCode,
    },
    /// timed out after {after_ms}ms waiting for {method} notification; set RUST_LOG=lspresso=debug for a message trace
    NotificationTimeout { method: String, after_ms: u64 },
    /// server closed the connection{exit_code} while waiting for {method} notification
    NotificationDisconnected { method: String, exit_code: ExitCode },
    /// server closed the connection{exit_code} while sending {method}
    SendDisconnected { method: String, exit_code: ExitCode },
    /// timed out after {after_ms}ms waiting for diagnostics to settle ({uri}); set RUST_LOG=lspresso=debug for a message trace
    DiagnosticsSettleTimeout { uri: Uri, after_ms: u64 },
    /// timed out after {after_ms}ms waiting for diagnostics to clear ({uri}); set RUST_LOG=lspresso=debug for a message trace
    DiagnosticsClearedTimeout { uri: Uri, after_ms: u64 },
    /// expected no diagnostics for {uri} but received {count}
    UnexpectedDiagnostics { uri: Uri, count: usize },
    /// document not open ({uri}): call did_open first
    NotOpen { uri: Uri },
    /// document already open ({uri}): call did_close before opening again
    AlreadyOpen { uri: Uri },
    /// client already shut down: call spawn/spawn_initialized for a new session
    AlreadyShutdown,
    /// not yet initialized: call initialize before sending protocol requests
    NotInitialized,
    /// already initialized: call initialize only once per session
    AlreadyInitialized,
    /// response id {id} doesn't match any request lspresso sent (a bogus or duplicate id from the server)
    UnmatchedResponseId { id: String },
}

#[derive(Debug)]
pub(crate) enum DisconnectKind {
    Send { method: String },
    Response { method: String, id: i32 },
    Notification { method: String },
}

pub(crate) fn elapsed_ms(start: &Instant) -> u64 {
    start.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

impl Error {
    pub(crate) fn serialize_error(source: serde_json::Error, method: &str) -> Self {
        ErrorKind::SerializeError {
            source,
            method: method.to_owned(),
        }
        .into()
    }

    pub(crate) fn deserialize_error(
        source: serde_json::Error,
        method: &str,
        raw: serde_json::Value,
    ) -> Self {
        ErrorKind::DeserializeError {
            source,
            method: method.to_owned(),
            raw,
        }
        .into()
    }

    pub(crate) fn diagnostics_cleared_timeout(uri: Uri, start: &Instant) -> Self {
        ErrorKind::DiagnosticsClearedTimeout {
            uri,
            after_ms: elapsed_ms(start),
        }
        .into()
    }

    pub(crate) fn unexpected_diagnostics(uri: Uri, count: usize) -> Self {
        ErrorKind::UnexpectedDiagnostics { uri, count }.into()
    }

    pub(crate) fn is_notification_deadline(&self) -> bool {
        matches!(
            &*self.0,
            ErrorKind::NotificationTimeout { .. } | ErrorKind::NotificationDisconnected { .. }
        )
    }

    pub(crate) fn is_response_deadline(&self) -> bool {
        matches!(
            &*self.0,
            ErrorKind::ResponseTimeout { .. } | ErrorKind::ResponseDisconnected { .. }
        )
    }

    pub(crate) fn into_diag_timeout(
        self,
        uri: Uri,
        after_ms: u64,
        make: impl FnOnce(Uri, u64) -> ErrorKind,
    ) -> Self {
        if self.is_notification_deadline() {
            make(uri, after_ms).into()
        } else {
            self
        }
    }

    pub(crate) fn disconnected(context: DisconnectKind, exit_code: Option<i32>) -> Self {
        let exit_code = ExitCode(exit_code);
        match context {
            DisconnectKind::Send { method } => {
                ErrorKind::SendDisconnected { method, exit_code }.into()
            }
            DisconnectKind::Response { method, id } => ErrorKind::ResponseDisconnected {
                method,
                id,
                exit_code,
            }
            .into(),
            DisconnectKind::Notification { method } => {
                ErrorKind::NotificationDisconnected { method, exit_code }.into()
            }
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
