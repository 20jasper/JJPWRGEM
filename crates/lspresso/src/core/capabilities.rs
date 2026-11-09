//! editor capabilities captured from real editors

use gen_lsp_types::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(capabilities: serde_json::Value) -> ServerCapabilities {
        serde_json::from_value(capabilities).unwrap()
    }

    #[test]
    fn save_includes_text_only_when_the_server_asks_for_it() {
        assert!(!save_includes_text(&declared(serde_json::json!({}))));
        assert!(!save_includes_text(&declared(serde_json::json!({
            "textDocumentSync": { "save": true }
        }))));
        assert!(!save_includes_text(&declared(serde_json::json!({
            "textDocumentSync": { "save": { "includeText": false } }
        }))));
        assert!(save_includes_text(&declared(serde_json::json!({
            "textDocumentSync": { "save": { "includeText": true } }
        }))));
    }
}
