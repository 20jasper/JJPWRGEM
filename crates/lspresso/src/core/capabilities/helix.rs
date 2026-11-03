use gen_lsp_types::ClientCapabilities;

use super::from_capture;

/// The newest captured release.
pub fn latest() -> ClientCapabilities {
    v25_07_1()
}

/// Helix 25.07.1, as captured from a real instance.
pub fn v25_07_1() -> ClientCapabilities {
    from_capture(include_str!("helix/25.07.1.json"))
}

/// Helix 24.07, as captured from a real instance.
pub fn v24_07() -> ClientCapabilities {
    from_capture(include_str!("helix/24.07.json"))
}

/// Helix 23.10, as captured from a real instance.
pub fn v23_10() -> ClientCapabilities {
    from_capture(include_str!("helix/23.10.json"))
}
