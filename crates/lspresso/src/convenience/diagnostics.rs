use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

impl Client {
    /// Pulls the diagnostics for `uri`.
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

    /// Pulls the diagnostics for the whole workspace.
    pub fn workspace_diagnostic(
        &mut self,
        params: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReport> {
        self.protocol_workspace_diagnostic(params)
    }
}
