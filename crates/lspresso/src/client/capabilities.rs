use gen_lsp_types::{ClientCapabilities, InitializeParams, ServerCapabilities};

use super::Client;

pub(crate) fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
    InitializeParams {
        capabilities,
        process_id: Some(i32::try_from(std::process::id()).unwrap()),
        ..Default::default()
    }
}

// lspclient[impl init-capabilities-gate-features]
impl Client {
    pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.lifecycle.capabilities()
    }
}
