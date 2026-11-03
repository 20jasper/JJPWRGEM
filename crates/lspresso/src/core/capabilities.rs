//! editor capabilities captured from real editors

use gen_lsp_types::{ClientCapabilities, InitializeParams};

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
