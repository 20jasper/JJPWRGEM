//! Client capability presets, and answers to "does the server support X"
//! from the capabilities it declared. Presets declare only what lspresso
//! honors -- position encodings, in each editor's order of preference --
//! and no pull diagnostics, since lspresso waits for pushed ones.

use gen_lsp_types::{
    ClientCapabilities, CodeActionProvider, DocumentFormattingProvider, GeneralClientCapabilities,
    InitializeParams, PositionEncodingKind, ServerCapabilities, TextDocumentSync,
    TextDocumentSyncKind,
};

fn with_position_encodings(position_encodings: Vec<PositionEncodingKind>) -> ClientCapabilities {
    ClientCapabilities {
        general: Some(GeneralClientCapabilities {
            position_encodings: Some(position_encodings),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// vscode-languageclient 10.1.0
/// <https://github.com/microsoft/vscode-languageserver-node/blob/0da5527ddf79ec7b10c953a1b1e6e007c9678326/client/src/common/client.ts#L2102-L2154>
pub fn vscode() -> ClientCapabilities {
    with_position_encodings(vec![PositionEncodingKind::UTF16])
}

/// Helix 25.07.1
/// <https://github.com/helix-editor/helix/blob/a05c151bb6e8e9c65ec390b0ae2afe7a5efd619b/helix-lsp/src/client.rs#L553-L722>
pub fn helix() -> ClientCapabilities {
    with_position_encodings(vec![
        PositionEncodingKind::UTF8,
        PositionEncodingKind::UTF32,
        PositionEncodingKind::UTF16,
    ])
}

/// Neovim master
/// <https://github.com/neovim/neovim/blob/12ea9735f77b81f0fecc7abe712639f88889f7f3/runtime/lua/vim/lsp/protocol.lua#L343-L646>
pub fn neovim() -> ClientCapabilities {
    with_position_encodings(vec![
        PositionEncodingKind::UTF8,
        PositionEncodingKind::UTF16,
        PositionEncodingKind::UTF32,
    ])
}

/// Zed 1.10.0
/// <https://github.com/zed-industries/zed/blob/550ddc9405943cfd69f34646f7af0179a5b0be41/crates/lsp/src/lsp.rs#L746-L1054>
pub fn zed() -> ClientCapabilities {
    with_position_encodings(vec![PositionEncodingKind::UTF16])
}

pub(crate) fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
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

/// An explicit `false` means unsupported, not just an absent field.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(capabilities: serde_json::Value) -> ServerCapabilities {
        serde_json::from_value(capabilities).unwrap()
    }

    #[test]
    fn initialize_params_carries_the_real_process_id_and_given_capabilities() {
        let params = initialize_params(vscode());
        assert_eq!(
            params.process_id,
            Some(i32::try_from(std::process::id()).unwrap())
        );
        assert_eq!(params.capabilities, vscode());
    }

    #[test]
    fn an_explicit_false_provider_is_unsupported() {
        let capabilities = declared(serde_json::json!({
            "documentFormattingProvider": false,
            "codeActionProvider": false,
        }));
        assert!(!supports_formatting(&capabilities));
        assert!(!supports_code_action(&capabilities));
    }

    #[test]
    fn a_provider_declared_with_options_is_supported() {
        let capabilities = declared(serde_json::json!({
            "documentFormattingProvider": {},
            "codeActionProvider": { "codeActionKinds": ["quickfix"] },
        }));
        assert!(supports_formatting(&capabilities));
        assert!(supports_code_action(&capabilities));
    }

    #[test]
    fn nothing_declared_supports_nothing() {
        let capabilities = declared(serde_json::json!({}));
        assert!(!supports_formatting(&capabilities));
        assert!(!supports_code_action(&capabilities));
        assert!(!supports_pull_diagnostics(&capabilities));
        assert_eq!(text_document_sync_kind(&capabilities), None);
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
