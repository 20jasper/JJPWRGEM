use gen_lsp_types::{
    ClientCapabilities, DiagnosticClientCapabilities, GeneralClientCapabilities, InitializeParams,
    PositionEncodingKind, ServerCapabilities, TextDocumentClientCapabilities,
};

use super::Client;

fn build_capabilities(
    position_encodings: Vec<PositionEncodingKind>,
    diagnostic: Option<DiagnosticClientCapabilities>,
) -> ClientCapabilities {
    ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities {
            diagnostic,
            ..Default::default()
        }),
        general: Some(GeneralClientCapabilities {
            position_encodings: Some(position_encodings),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// `dynamicRegistration` is deliberately always `false` here, even for
// presets whose real editor declares `true` -- lspresso answers
// `client/registerCapability` with a generic, unrecorded `null`, so a
// server that skips declaring `diagnosticProvider` statically and instead
// registers it dynamically (which declaring `true` invites) would go
// unnoticed by a future `supports_pull_diagnostics()`. Advertising `true`
// here would be lying about what this client can actually honor.
fn build_pull_diagnostics(related_document_support: bool) -> DiagnosticClientCapabilities {
    DiagnosticClientCapabilities {
        dynamic_registration: Some(false),
        related_document_support: Some(related_document_support),
        ..Default::default()
    }
}

pub(crate) fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
    InitializeParams {
        capabilities,
        process_id: Some(
            i32::try_from(std::process::id()).expect("pid fits in i32 on all supported OSes"),
        ),
        ..Default::default()
    }
}

impl Client {
    pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.lifecycle.capabilities()
    }

    // lspclient[impl cap-missing-means-absent]
    // `is_some_and` treats a missing field the same as an explicit "not
    // supported" -- there's no separate "try it anyway" path for an absent
    // capability.
    pub fn supports_pull_diagnostics(&self) -> bool {
        self.server_capabilities()
            .is_some_and(|caps| caps.diagnostic_provider.is_some())
    }
}

/// vscode-languageclient 10.1.0
/// <https://github.com/microsoft/vscode-languageserver-node/blob/0da5527ddf79ec7b10c953a1b1e6e007c9678326/client/src/common/client.ts#L2102-L2154>
/// <https://github.com/microsoft/vscode-languageserver-node/blob/0da5527ddf79ec7b10c953a1b1e6e007c9678326/client/src/common/diagnostic.ts#L1115-L1131>
pub fn vscode() -> ClientCapabilities {
    build_capabilities(
        vec![PositionEncodingKind::UTF16],
        Some(build_pull_diagnostics(false)),
    )
}

#[cfg(test)]
mod vscode_tests {
    use super::vscode;

    #[test]
    fn declares_utf16_and_pull_diagnostics_without_dynamic_registration() {
        let caps = vscode();

        let encodings = caps
            .general
            .expect("general capabilities declared")
            .position_encodings
            .expect("position encodings declared");
        assert_eq!(encodings, vec![gen_lsp_types::PositionEncodingKind::UTF16]);

        let diagnostic = caps
            .text_document
            .expect("text document capabilities declared")
            .diagnostic
            .expect("pull diagnostics declared");
        assert_eq!(diagnostic.dynamic_registration, Some(false));
        assert_eq!(diagnostic.related_document_support, Some(false));
    }
}
