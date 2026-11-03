//! Client capability presets. Presets declare position encodings in each
//! editor's order of preference

use gen_lsp_types::{
    ClientCapabilities, GeneralClientCapabilities, InitializeParams, PositionEncodingKind,
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

fn from_capture(capabilities: &str) -> ClientCapabilities {
    serde_json::from_str(capabilities).expect("captured capabilities are valid")
}

/// Helix 25.07.1, as captured from a real instance.
///
/// Declares `dynamicRegistration` for `workspace.didChangeWatchedFiles`,
/// which lspresso doesn't honor yet: it answers `client/registerCapability`
/// with a bare `null` and doesn't record the registration.
pub fn helix() -> ClientCapabilities {
    from_capture(include_str!("capabilities/helix-25.07.1.json"))
}

/// Neovim 0.12.6, as captured from a real instance.
///
/// Declares `dynamicRegistration` for diagnostic, formatting, range
/// formatting, inlay hint, definition, color provider, rename, code action,
/// and hover, which lspresso doesn't honor yet: it answers
/// `client/registerCapability` with a bare `null` and doesn't record the
/// registration.
pub fn neovim() -> ClientCapabilities {
    from_capture(include_str!("capabilities/neovim-0.12.6.json"))
}

/// Zed 1.10.0
/// <https://github.com/zed-industries/zed/blob/550ddc9405943cfd69f34646f7af0179a5b0be41/crates/lsp/src/lsp.rs#L746-L1054>
pub fn zed() -> ClientCapabilities {
    with_position_encodings(vec![PositionEncodingKind::UTF16])
}

pub fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
    InitializeParams {
        process_id: i32::try_from(std::process::id()).ok(),
        capabilities,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn captured_presets_keep_every_field_the_editor_sent() {
        let neovim_json: Value =
            serde_json::from_str(include_str!("capabilities/neovim-0.12.6.json")).unwrap();
        let helix_json: Value =
            serde_json::from_str(include_str!("capabilities/helix-25.07.1.json")).unwrap();

        assert_eq!(serde_json::to_value(neovim()).unwrap(), neovim_json);
        assert_eq!(serde_json::to_value(helix()).unwrap(), helix_json);
    }
}
