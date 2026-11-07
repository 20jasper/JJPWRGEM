//! editor capabilities captured from real editors

use gen_lsp_types::{
    ClientCapabilities, InitializeParams, ServerCapabilities, TextDocumentSync,
    TextDocumentSyncKind,
};

pub mod emacs;
pub mod helix;
pub mod neovim;
pub mod vscode;

fn from_capture(capabilities: &str) -> ClientCapabilities {
    serde_json::from_str(capabilities).expect("captured capabilities are valid")
}

pub fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
    InitializeParams {
        process_id: i32::try_from(std::process::id()).ok(),
        capabilities,
        ..Default::default()
    }
}

pub(crate) fn text_document_sync_kind(
    capabilities: &ServerCapabilities,
) -> Option<TextDocumentSyncKind> {
    match capabilities.text_document_sync.as_ref()? {
        TextDocumentSync::Kind(kind) => Some(*kind),
        TextDocumentSync::Options(options) => options.change,
    }
}
