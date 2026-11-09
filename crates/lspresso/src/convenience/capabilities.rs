use gen_lsp_types::*;

use crate::{Client, core};

impl Client {
    pub fn text_document_sync_kind(&self) -> Option<TextDocumentSyncKind> {
        self.server_capabilities()
            .and_then(core::capabilities::text_document_sync_kind)
    }

    pub fn supports_incremental_sync(&self) -> bool {
        self.text_document_sync_kind() == Some(TextDocumentSyncKind::Incremental)
    }

    pub fn supports_save_text(&self) -> bool {
        self.server_capabilities()
            .is_some_and(core::capabilities::save_includes_text)
    }

    pub fn supports_pull_diagnostics(&self) -> bool {
        self.server_capabilities()
            .is_some_and(|capabilities| capabilities.diagnostic_provider.is_some())
    }
}
