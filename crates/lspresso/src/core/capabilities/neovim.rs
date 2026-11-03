use gen_lsp_types::ClientCapabilities;

use super::from_capture;

/// The newest captured release.
pub fn latest() -> ClientCapabilities {
    v0_12_6()
}

/// Neovim 0.12.6, as captured from a real instance.
pub fn v0_12_6() -> ClientCapabilities {
    from_capture(include_str!("neovim/v0.12.6.json"))
}

/// Neovim 0.11.5, as captured from a real instance.
pub fn v0_11_5() -> ClientCapabilities {
    from_capture(include_str!("neovim/v0.11.5.json"))
}

/// Neovim 0.10.2, as captured from a real instance.
pub fn v0_10_2() -> ClientCapabilities {
    from_capture(include_str!("neovim/v0.10.2.json"))
}

/// Neovim 0.9.4, as captured from a real instance.
pub fn v0_9_4() -> ClientCapabilities {
    from_capture(include_str!("neovim/v0.9.4.json"))
}
