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

/// `None` when the server declared no sync kind, in either the bare-kind
/// or the options form of `textDocumentSync`.
pub(crate) fn text_document_sync_kind(
    capabilities: &ServerCapabilities,
) -> Option<TextDocumentSyncKind> {
    match capabilities.text_document_sync.as_ref()? {
        TextDocumentSync::Kind(kind) => Some(*kind),
        TextDocumentSync::Options(options) => options.change,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(capabilities: serde_json::Value) -> ServerCapabilities {
        serde_json::from_value(capabilities).unwrap()
    }

    #[test]
    fn nothing_declared_means_no_sync_kind() {
        assert_eq!(
            text_document_sync_kind(&declared(serde_json::json!({}))),
            None
        );
    }

    #[test]
    fn sync_kind_is_read_from_either_form() {
        let bare = declared(serde_json::json!({ "textDocumentSync": 2 }));
        assert_eq!(
            text_document_sync_kind(&bare),
            Some(TextDocumentSyncKind::Incremental)
        );
        let options = declared(serde_json::json!({ "textDocumentSync": { "change": 1 } }));
        assert_eq!(
            text_document_sync_kind(&options),
            Some(TextDocumentSyncKind::Full)
        );
        let options_without_change =
            declared(serde_json::json!({ "textDocumentSync": { "openClose": true } }));
        assert_eq!(text_document_sync_kind(&options_without_change), None);
    }
}
