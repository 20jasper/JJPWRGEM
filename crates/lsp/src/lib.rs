mod backend;
mod lsp_server;
mod range;
mod server;
#[cfg(not(target_arch = "wasm32"))]
mod stdio;
mod transport;

pub use server::Server;
#[cfg(not(target_arch = "wasm32"))]
pub use stdio::run_blocking;
