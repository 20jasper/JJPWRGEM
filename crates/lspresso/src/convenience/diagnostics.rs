use std::time::{Duration, Instant};

use gen_lsp_types::*;
use lsp_server::Notification as LspNotification;

use super::document;
use crate::{
    Client, Protocol, Result,
    core::{self, inbox::Verdict},
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

    /// Pulls when supported or waits for pushed diagnostics.
    pub fn diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        self.require_open_document(uri)?;
        if self.supports_pull_diagnostics() {
            self.pull_diagnostics(uri)
        } else {
            self.next_diagnostics(uri, self.request_timeout())
        }
    }

    pub fn next_diagnostics(&mut self, uri: &Uri, timeout: Duration) -> Result<Vec<Diagnostic>> {
        let start = Instant::now();
        let deadline = start + timeout;
        let mut judge = self.diagnostics_judge(uri)?;
        if let Some(diagnostics) = self.take_notification(&mut judge) {
            return Ok(diagnostics);
        }
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(ErrorKind::DiagnosticsTimeout {
                    uri: uri.clone(),
                    after_ms: elapsed_ms(start, now),
                }
                .into());
            }
            self.receive_until(deadline, PublishDiagnosticsNotification::METHOD.as_str())?;
            if let Some(diagnostics) = self.take_notification(&mut judge) {
                return Ok(diagnostics);
            }
        }
    }

    fn diagnostics_judge(
        &self,
        uri: &Uri,
    ) -> Result<impl FnMut(&LspNotification) -> Verdict<Vec<Diagnostic>> + use<>> {
        let version = self.open_document_version(uri)?;
        let uri = uri.clone();
        Ok(move |notification: &LspNotification| {
            core::diagnostics::publication(&uri, version, notification)
        })
    }

    fn pull_diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        let mut attempt = 1;
        loop {
            let result = self.protocol_diagnostic(DocumentDiagnosticParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                identifier: None,
                previous_result_id: self.diagnostic_cache.previous_result_id(uri),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            });
            match result {
                Ok(report) => return Ok(self.diagnostic_cache.apply(uri, report)),
                Err(err)
                    if attempt < core::diagnostics::MAX_PULL_ATTEMPTS
                        && core::diagnostics::should_retry_pull(err.kind()) =>
                {
                    attempt += 1;
                }
                Err(err) => return Err(err),
            }
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
