use gen_lsp_types::*;

use super::document;
use crate::{Client, Protocol, Result};

fn whole_document(text: String) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeWholeDocument { text }.into()
}

impl Client {
    pub fn did_open(
        &mut self,
        uri: Uri,
        text: impl Into<String>,
        language: LanguageKind,
    ) -> Result<()> {
        let version = self.next_open_version(&uri);
        self.protocol_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id: language,
                version,
                text: text.into(),
            },
        })
    }

    pub fn did_change_full(&mut self, uri: &Uri, text: impl Into<String>) -> Result<()> {
        let version = self.next_document_version(uri)?;
        let text = text.into();
        let content_changes = vec![whole_document(text.clone())];
        self.send_validated_change(uri, version, content_changes, text)
    }

    /// Sends edits as incremental or whole-document changes.
    pub fn did_change(&mut self, uri: &Uri, edits: &[TextEdit]) -> Result<()> {
        if edits.is_empty() {
            self.require_open_document(uri)?;
            return Ok(());
        }
        let (version, text) = self.next_document_version_and_text(uri)?;
        let applied = self.apply_edits(text, edits)?;
        let content_changes = if self.supports_incremental_sync() {
            applied.changes
        } else {
            vec![whole_document(applied.text.clone())]
        };
        self.send_validated_change(uri, version, content_changes, applied.text)
    }

    pub fn did_close(&mut self, uri: &Uri) -> Result<()> {
        self.protocol_did_close(DidCloseTextDocumentParams {
            text_document: document(uri),
        })
    }

    pub fn document_text(&self, uri: &Uri) -> Option<&str> {
        self.tracked_text(uri).ok()
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
    pub fn did_save(&mut self, uri: &Uri) -> Result<()> {
        let text = self
            .supports_save_text()
            .then(|| self.tracked_text(uri).map(str::to_owned))
            .transpose()?;
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
