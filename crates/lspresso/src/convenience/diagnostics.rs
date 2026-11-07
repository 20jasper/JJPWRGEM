use std::time::{Duration, Instant};

use gen_lsp_types::*;

use super::document;
use crate::{
    Client, Protocol, Result,
    error::{ErrorKind, elapsed_ms},
};

impl Client {
    /// Sends [`textDocument/diagnostic`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_diagnostic).
    ///
    /// Use [`Protocol::protocol_diagnostic`] for full params.
    pub fn diagnostic(&mut self, uri: &Uri) -> Result<DocumentDiagnosticReport> {
        self.protocol_diagnostic(DocumentDiagnosticParams {
            text_document: document(uri),
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Waits for the diagnostics the server publishes for the open document's
    /// current version, for as long as a request may take.
    pub fn diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        self.next_diagnostics(uri, self.request_timeout())
    }

    pub fn next_diagnostics(&mut self, uri: &Uri, timeout: Duration) -> Result<Vec<Diagnostic>> {
        let start = Instant::now();
        let deadline = start + timeout;
        let version = self.open_document_version(uri)?;
        loop {
            if let Some(diagnostics) = self.take_publication(uri, version) {
                return Ok(diagnostics);
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(ErrorKind::DiagnosticsTimeout {
                    uri: uri.clone(),
                    after_ms: elapsed_ms(start, now),
                }
                .into());
            }
            self.receive_until(deadline, PublishDiagnosticsNotification::METHOD.as_str())?;
        }
    }

    /// Sends [`workspace/diagnostic`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_diagnostic).
    ///
    /// Use [`Protocol::protocol_workspace_diagnostic`] for full params.
    pub fn workspace_diagnostic(
        &mut self,
        params: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReport> {
        self.protocol_workspace_diagnostic(params)
    }
}
