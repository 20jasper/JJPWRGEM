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

    pub fn supports_pull_diagnostics(&self) -> bool {
        self.server_supports_pull_diagnostics()
    }

    /// Pulls when supported or waits for pushed diagnostics.
    pub fn diagnostics(&mut self, uri: &Uri) -> Result<Vec<Diagnostic>> {
        self.diagnostics_for(uri)
    }

    pub fn next_diagnostics(&mut self, uri: &Uri, timeout: Duration) -> Result<Vec<Diagnostic>> {
        self.wait_for_pushed_diagnostics(uri, timeout)
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
