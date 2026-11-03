//! Client capability presets. Each editor has a module with `latest()` and one
//! function per captured release. Zed has no capture yet, so it only declares
//! position encodings in its order of preference.

use gen_lsp_types::{
    ClientCapabilities, GeneralClientCapabilities, InitializeParams, PositionEncodingKind,
};

pub mod emacs;
pub mod helix;
pub mod neovim;
pub mod vscode;

fn with_position_encodings(position_encodings: Vec<PositionEncodingKind>) -> ClientCapabilities {
    ClientCapabilities {
        general: Some(GeneralClientCapabilities {
            position_encodings: Some(position_encodings),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn from_capture(capabilities: &str) -> ClientCapabilities {
    serde_json::from_str(capabilities).expect("captured capabilities are valid")
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
