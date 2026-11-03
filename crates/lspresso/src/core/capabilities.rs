//! editor capabilities captured from real editors

use gen_lsp_types::{
    ClientCapabilities, CodeActionProvider, DocumentFormattingProvider, InitializeParams, Save,
    SaveOptions, ServerCapabilities, TextDocumentSync, TextDocumentSyncKind,
    TextDocumentSyncOptions,
};

pub mod emacs;
pub mod helix;
pub mod neovim;
pub mod vscode;

fn from_capture(capabilities: &str) -> ClientCapabilities {
    serde_json::from_str(capabilities).expect("captured capabilities are valid")
}

pub(crate) fn text_document_sync_kind(
    capabilities: &ServerCapabilities,
) -> Option<TextDocumentSyncKind> {
    match capabilities.text_document_sync.as_ref()? {
        TextDocumentSync::Kind(kind) => Some(*kind),
        TextDocumentSync::Options(options) => options.change,
    }
}

pub(crate) fn save_includes_text(capabilities: &ServerCapabilities) -> bool {
    matches!(
        &capabilities.text_document_sync,
        Some(TextDocumentSync::Options(TextDocumentSyncOptions {
            save: Some(Save::SaveOptions(SaveOptions {
                include_text: Some(true)
            })),
            ..
        }))
    )
}

pub(crate) fn supports_formatting(capabilities: &ServerCapabilities) -> bool {
    match capabilities.document_formatting_provider {
        Some(DocumentFormattingProvider::Bool(supported)) => supported,
        Some(DocumentFormattingProvider::DocumentFormattingOptions(_)) => true,
        None => false,
    }
}

pub(crate) fn supports_code_action(capabilities: &ServerCapabilities) -> bool {
    match capabilities.code_action_provider {
        Some(CodeActionProvider::Bool(supported)) => supported,
        Some(CodeActionProvider::CodeActionOptions(_)) => true,
        None => false,
    }
}

pub(crate) const fn supports_pull_diagnostics(capabilities: &ServerCapabilities) -> bool {
    capabilities.diagnostic_provider.is_some()
}

pub fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
    InitializeParams {
        process_id: i32::try_from(std::process::id()).ok(),
        capabilities,
        ..Default::default()
    }
}
