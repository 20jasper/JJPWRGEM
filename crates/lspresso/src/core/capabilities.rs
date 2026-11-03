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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn assert_preset_matches_capture(name: &str, preset: ClientCapabilities, capture: &str) {
        let captured: Value = serde_json::from_str(capture).unwrap();
        assert_eq!(serde_json::to_value(preset).unwrap(), captured, "{name}");
    }

    #[test]
    fn captured_presets_keep_every_field_the_editor_sent() {
        assert_preset_matches_capture(
            "neovim/v0.12.6.json",
            neovim::v0_12_6(),
            include_str!("capabilities/neovim/v0.12.6.json"),
        );
        assert_preset_matches_capture(
            "neovim/v0.11.5.json",
            neovim::v0_11_5(),
            include_str!("capabilities/neovim/v0.11.5.json"),
        );
        assert_preset_matches_capture(
            "neovim/v0.10.2.json",
            neovim::v0_10_2(),
            include_str!("capabilities/neovim/v0.10.2.json"),
        );
        assert_preset_matches_capture(
            "helix/25.07.1.json",
            helix::v25_07_1(),
            include_str!("capabilities/helix/25.07.1.json"),
        );
        assert_preset_matches_capture(
            "helix/24.07.json",
            helix::v24_07(),
            include_str!("capabilities/helix/24.07.json"),
        );
        assert_preset_matches_capture(
            "helix/23.10.json",
            helix::v23_10(),
            include_str!("capabilities/helix/23.10.json"),
        );
        assert_preset_matches_capture(
            "vscode/11.0.0.json",
            vscode::v11_0_0(),
            include_str!("capabilities/vscode/11.0.0.json"),
        );
        assert_preset_matches_capture(
            "vscode/9.0.1.json",
            vscode::v9_0_1(),
            include_str!("capabilities/vscode/9.0.1.json"),
        );
        assert_preset_matches_capture(
            "vscode/8.1.0.json",
            vscode::v8_1_0(),
            include_str!("capabilities/vscode/8.1.0.json"),
        );
        assert_preset_matches_capture(
            "emacs/30.2.json",
            emacs::v30_2(),
            include_str!("capabilities/emacs/30.2.json"),
        );
        assert_preset_matches_capture(
            "emacs/29.4.json",
            emacs::v29_4(),
            include_str!("capabilities/emacs/29.4.json"),
        );
        assert_preset_matches_capture(
            "emacs/29.1.json",
            emacs::v29_1(),
            include_str!("capabilities/emacs/29.1.json"),
        );
    }

    #[test]
    fn neovim_0_9_4_drops_only_the_workspace_symbol_hierarchy_flag() {
        let mut captured: Value =
            serde_json::from_str(include_str!("capabilities/neovim/v0.9.4.json")).unwrap();
        captured["workspace"]["symbol"]
            .as_object_mut()
            .unwrap()
            .remove("hierarchicalWorkspaceSymbolSupport");

        assert_eq!(serde_json::to_value(neovim::v0_9_4()).unwrap(), captured);
    }

    #[test]
    fn emacs_31_1_drops_only_the_streaming_diagnostics_flag() {
        let mut captured: Value =
            serde_json::from_str(include_str!("capabilities/emacs/31.1.json")).unwrap();
        captured["textDocument"]
            .as_object_mut()
            .unwrap()
            .remove("$streamingDiagnostics");

        assert_eq!(serde_json::to_value(emacs::v31_1()).unwrap(), captured);
    }

    #[test]
    fn latest_is_the_newest_captured_release() {
        assert_eq!(
            serde_json::to_value(neovim::latest()).unwrap(),
            serde_json::to_value(neovim::v0_12_6()).unwrap(),
        );
        assert_eq!(
            serde_json::to_value(helix::latest()).unwrap(),
            serde_json::to_value(helix::v25_07_1()).unwrap(),
        );
        assert_eq!(
            serde_json::to_value(vscode::latest()).unwrap(),
            serde_json::to_value(vscode::v11_0_0()).unwrap(),
        );
        assert_eq!(
            serde_json::to_value(emacs::latest()).unwrap(),
            serde_json::to_value(emacs::v31_1()).unwrap(),
        );
    }
}
