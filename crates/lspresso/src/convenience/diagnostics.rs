use std::time::Duration;

use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result, core};

impl Client {
    /// Sends [`textDocument/diagnostic`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_diagnostic).
    ///
    /// Use [`Protocol::protocol_diagnostic`] for full params.
    pub fn diagnostic(&mut self, uri: &Uri) -> Result<DocumentDiagnosticReport> {
        self.protocol_diagnostic(diagnostic_params(uri, None))
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
        let version = self.open_document_version(uri)?;
        let published = self
            .wait_for_notification_where::<PublishDiagnosticsNotification>(timeout, |p| {
                p.uri == *uri && p.version.is_none_or(|v| v >= version)
            })?;
        Ok(published.diagnostics)
    }

    fn pull_diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        let mut attempt = 1;
        loop {
            let previous_result_id = self.previous_result_id(uri)?;
            let result = self.protocol_diagnostic(diagnostic_params(uri, previous_result_id));
            match result {
                Ok(report) => return self.apply_pulled_report(uri, report),
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

fn diagnostic_params(uri: &Uri, previous_result_id: Option<String>) -> DocumentDiagnosticParams {
    DocumentDiagnosticParams {
        text_document: document(uri),
        identifier: None,
        previous_result_id,
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}
