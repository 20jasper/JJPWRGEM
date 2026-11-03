//! Emacs presets, one per captured release. Each is Eglot, the LSP client
//! bundled with that Emacs version.

use gen_lsp_types::ClientCapabilities;

use super::from_capture;

/// The newest captured release.
pub fn latest() -> ClientCapabilities {
    v31_1()
}

/// Emacs 31.1 (Eglot), as captured from a source build.
pub fn v31_1() -> ClientCapabilities {
    from_capture(include_str!("emacs/31.1.json"))
}

/// Emacs 30.2 (Eglot 1.17.30), as captured from a Fedora build.
pub fn v30_2() -> ClientCapabilities {
    from_capture(include_str!("emacs/30.2.json"))
}

/// Emacs 29.4 (Eglot), as captured from a source build.
pub fn v29_4() -> ClientCapabilities {
    from_capture(include_str!("emacs/29.4.json"))
}

/// Emacs 29.1 (Eglot), as captured from a source build.
pub fn v29_1() -> ClientCapabilities {
    from_capture(include_str!("emacs/29.1.json"))
}
