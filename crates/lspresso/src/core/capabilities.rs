//! The `initialize` params a client sends.

use gen_lsp_types::{ClientCapabilities, InitializeParams};

pub(crate) fn initialize_params(capabilities: ClientCapabilities) -> InitializeParams {
    InitializeParams {
        process_id: i32::try_from(std::process::id()).ok(),
        capabilities,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use gen_lsp_types::{GeneralClientCapabilities, PositionEncodingKind};

    use super::*;

    #[test]
    fn initialize_params_carries_the_real_process_id_and_given_capabilities() {
        let capabilities = ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: Some(vec![PositionEncodingKind::UTF8]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let params = initialize_params(capabilities.clone());
        assert_eq!(
            params.process_id,
            Some(i32::try_from(std::process::id()).unwrap())
        );
        assert_eq!(params.capabilities, capabilities);
    }
}
