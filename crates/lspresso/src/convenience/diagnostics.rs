use std::time::Duration;

use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

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
        let version = self.open_document_version(uri)?;
        let published = self
            .wait_for_notification_where::<PublishDiagnosticsNotification>(timeout, |p| {
                p.uri == *uri && p.version.is_none_or(|v| v >= version)
            })?;
        Ok(published.diagnostics)
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
