use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

impl Client {
    /// Lists the symbols in `uri`.
    ///
    /// Use [`Protocol::protocol_document_symbol`] for full params.
    pub fn document_symbol(&mut self, uri: &Uri) -> Result<Option<DocumentSymbolResponse>> {
        self.protocol_document_symbol(DocumentSymbolParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Searches the workspace for symbols matching `query`.
    ///
    /// Use [`Protocol::protocol_workspace_symbol`] for full params.
    pub fn workspace_symbol(
        &mut self,
        query: impl Into<String>,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        self.protocol_workspace_symbol(WorkspaceSymbolParams {
            query: query.into(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Resolves the rest of a workspace `symbol`.
    pub fn workspace_symbol_resolve(&mut self, symbol: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        self.protocol_workspace_symbol_resolve(symbol)
    }
}
