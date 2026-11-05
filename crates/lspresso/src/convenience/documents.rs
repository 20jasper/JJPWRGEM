use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

impl Client {
    /// Notifies the server that `uri` is about to be saved for `reason`.
    ///
    /// Use [`Protocol::protocol_will_save`] for full params.
    pub fn will_save(&mut self, uri: &Uri, reason: TextDocumentSaveReason) -> Result<()> {
        self.protocol_will_save(WillSaveTextDocumentParams {
            text_document: document(uri),
            reason,
        })
    }

    /// Asks the server for edits to apply before `uri` is saved for `reason`.
    ///
    /// Use [`Protocol::protocol_will_save_wait_until`] for full params.
    pub fn will_save_wait_until(
        &mut self,
        uri: &Uri,
        reason: TextDocumentSaveReason,
    ) -> Result<Option<Vec<TextEdit>>> {
        self.protocol_will_save_wait_until(WillSaveTextDocumentParams {
            text_document: document(uri),
            reason,
        })
    }

    /// Notifies the server that `uri` was saved, optionally with its text.
    ///
    /// Use [`Protocol::protocol_did_save`] for full params.
    pub fn did_save(&mut self, uri: &Uri, text: Option<String>) -> Result<()> {
        self.protocol_did_save(DidSaveTextDocumentParams {
            text_document: document(uri),
            text,
        })
    }

    /// Notifies the server that a notebook document opened.
    pub fn notebook_document_did_open(
        &mut self,
        params: DidOpenNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_open(params)
    }

    /// Notifies the server that a notebook document changed.
    pub fn notebook_document_did_change(
        &mut self,
        params: DidChangeNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_change(params)
    }

    /// Notifies the server that a notebook document was saved.
    pub fn notebook_document_did_save(
        &mut self,
        params: DidSaveNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_save(params)
    }

    /// Notifies the server that a notebook document closed.
    pub fn notebook_document_did_close(
        &mut self,
        params: DidCloseNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_close(params)
    }
}
