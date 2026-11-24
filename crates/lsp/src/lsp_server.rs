// vendored from lsp-server v0.10.0
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod msg;
pub(crate) mod req_queue;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod stdio;
