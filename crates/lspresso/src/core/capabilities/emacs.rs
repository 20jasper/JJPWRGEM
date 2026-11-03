use gen_lsp_types::ClientCapabilities;

use super::from_capture;

// latest as of 2026-11-03
pub fn latest() -> ClientCapabilities {
    v31_1()
}

// Emacs 31.1 (Eglot)
pub fn v31_1() -> ClientCapabilities {
    from_capture(include_str!("emacs/31.1.json"))
}

// Emacs 30.2 (Eglot 1.17.30)
pub fn v30_2() -> ClientCapabilities {
    from_capture(include_str!("emacs/30.2.json"))
}

// Emacs 29.4 (Eglot)
pub fn v29_4() -> ClientCapabilities {
    from_capture(include_str!("emacs/29.4.json"))
}

// Emacs 29.1 (Eglot)
pub fn v29_1() -> ClientCapabilities {
    from_capture(include_str!("emacs/29.1.json"))
}
