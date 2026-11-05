use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

impl Client {
    /// Sends [`textDocument/documentSymbol`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_documentSymbol).
    ///
    /// Use [`Protocol::protocol_document_symbol`] for full params.
    pub fn document_symbol(&mut self, uri: &Uri) -> Result<Option<DocumentSymbolResponse>> {
        self.protocol_document_symbol(DocumentSymbolParams {
            text_document: document(uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }

    /// Sends [`workspace/symbol`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_symbol).
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

    /// Sends [`workspaceSymbol/resolve`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#workspace_symbolResolve).
    ///
    /// Use [`Protocol::protocol_workspace_symbol_resolve`] for full params.
    pub fn workspace_symbol_resolve(&mut self, symbol: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        self.protocol_workspace_symbol_resolve(symbol)
    }
}
