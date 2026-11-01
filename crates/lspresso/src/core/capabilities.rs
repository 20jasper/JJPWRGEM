//! Client capability presets. Presets declare only what lspresso
//! honors: position encodings, in each editor's order of preference.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_params_carries_the_real_process_id_and_given_capabilities() {
        let params = initialize_params(vscode());
        assert_eq!(
            params.process_id,
            Some(i32::try_from(std::process::id()).unwrap())
        );
        assert_eq!(params.capabilities, vscode());
    }
}
