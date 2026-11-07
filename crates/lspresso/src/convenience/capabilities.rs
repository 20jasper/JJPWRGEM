use gen_lsp_types::TextDocumentSyncKind;

use crate::{Client, core};

impl Client {
    pub fn text_document_sync_kind(&self) -> Option<TextDocumentSyncKind> {
        self.server_capabilities()
            .and_then(core::capabilities::text_document_sync_kind)
    }

    pub fn supports_incremental_sync(&self) -> bool {
        self.text_document_sync_kind() == Some(TextDocumentSyncKind::Incremental)
    }
}
