use gen_lsp_types::ClientCapabilities;

use super::from_capture;

// latest as of 2026-11-03
pub fn latest() -> ClientCapabilities {
    v11_0_0()
}

// vscode-languageclient 11.0.0
pub fn v11_0_0() -> ClientCapabilities {
    from_capture(include_str!("vscode/11.0.0.json"))
}

// vscode-languageclient 9.0.1
pub fn v9_0_1() -> ClientCapabilities {
    from_capture(include_str!("vscode/9.0.1.json"))
}

// vscode-languageclient 8.1.0
pub fn v8_1_0() -> ClientCapabilities {
    from_capture(include_str!("vscode/8.1.0.json"))
}
