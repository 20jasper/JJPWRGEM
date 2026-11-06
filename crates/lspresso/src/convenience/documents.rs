use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result, core};

impl Client {
    pub fn did_open(
        &mut self,
        uri: Uri,
        text: impl Into<String>,
        language: LanguageKind,
    ) -> Result<()> {
        self.protocol_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: language,
                version: 1,
                text: text.into(),
            },
        })
    }

    pub fn did_change_full(&mut self, uri: &Uri, text: impl Into<String>) -> Result<()> {
        let version = self.docs.next_version(uri)?;
        self.protocol_did_change(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                version,
                text_document_identifier: document(uri),
            },
            content_changes: vec![TextDocumentContentChangeEvent::from(
                TextDocumentContentChangeWholeDocument { text: text.into() },
            )],
        })
    }

    pub fn did_close(&mut self, uri: &Uri) -> Result<()> {
        self.protocol_did_close(DidCloseTextDocumentParams {
            text_document: document(uri),
        })
    }

    pub fn text_document_sync_kind(&self) -> Option<TextDocumentSyncKind> {
        self.server_capabilities()
            .and_then(core::capabilities::text_document_sync_kind)
    }

    pub fn supports_incremental_sync(&self) -> bool {
        self.text_document_sync_kind() == Some(TextDocumentSyncKind::Incremental)
    }

    /// Sends [`textDocument/willSave`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_willSave).
    ///
    /// Use [`Protocol::protocol_will_save`] for full params.
    pub fn will_save(&mut self, uri: &Uri, reason: TextDocumentSaveReason) -> Result<()> {
        self.protocol_will_save(WillSaveTextDocumentParams {
            text_document: document(uri),
            reason,
        })
    }

    /// Sends [`textDocument/willSaveWaitUntil`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_willSaveWaitUntil).
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

    /// Sends [`textDocument/didSave`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#textDocument_didSave).
    ///
    /// Use [`Protocol::protocol_did_save`] for full params.
    pub fn did_save(&mut self, uri: &Uri, text: Option<String>) -> Result<()> {
        self.protocol_did_save(DidSaveTextDocumentParams {
            text_document: document(uri),
            text,
        })
    }

    /// Sends [`notebookDocument/didOpen`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#notebookDocument_didOpen).
    ///
    /// Use [`Protocol::protocol_notebook_document_did_open`] for full params.
    pub fn notebook_document_did_open(
        &mut self,
        params: DidOpenNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_open(params)
    }

    /// Sends [`notebookDocument/didChange`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#notebookDocument_didChange).
    ///
    /// Use [`Protocol::protocol_notebook_document_did_change`] for full params.
    pub fn notebook_document_did_change(
        &mut self,
        params: DidChangeNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_change(params)
    }

    /// Sends [`notebookDocument/didSave`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#notebookDocument_didSave).
    ///
    /// Use [`Protocol::protocol_notebook_document_did_save`] for full params.
    pub fn notebook_document_did_save(
        &mut self,
        params: DidSaveNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_save(params)
    }

    /// Sends [`notebookDocument/didClose`](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/#notebookDocument_didClose).
    ///
    /// Use [`Protocol::protocol_notebook_document_did_close`] for full params.
    pub fn notebook_document_did_close(
        &mut self,
        params: DidCloseNotebookDocumentParams,
    ) -> Result<()> {
        self.protocol_notebook_document_did_close(params)
    }
}
