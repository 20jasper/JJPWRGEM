#[cfg(not(target_arch = "wasm32"))]
use std::process::ExitCode;

use gen_lsp_types::*;

use crate::{
    backend::{Backend, Lifecycle, SOURCE},
    range::PositionEncoding,
    transport::Sink,
};

fn client_supports_pull_diagnostics(capabilities: &ClientCapabilities) -> bool {
    capabilities
        .text_document
        .as_ref()
        .is_some_and(|text_document| text_document.diagnostic.is_some())
}

impl<S: Sink> Backend<S> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn exit_code(&self) -> ExitCode {
        if self.lifecycle == Lifecycle::ShutDown {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }

    pub(super) fn initialize(&mut self, params: InitializeParams) -> InitializeResult {
        self.lifecycle = Lifecycle::Running;
        self.position_encoding = PositionEncoding::negotiate(&params.capabilities);
        self.client_pulls_diagnostics = client_supports_pull_diagnostics(&params.capabilities);
        if let Some(options) = params.initialization_options {
            self.update_format_settings(options);
        }

        let position_encoding = self.position_encoding.advertised();
        InitializeResult {
            server_info: Some(ServerInfo {
                name: SOURCE.into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                position_encoding,
                text_document_sync: Some(TextDocumentSync::Options(TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::Incremental),
                    ..Default::default()
                })),
                document_formatting_provider: Some(true.into()),
                code_action_provider: Some(true.into()),
                document_symbol_provider: Some(true.into()),
                folding_range_provider: Some(true.into()),
                selection_range_provider: Some(true.into()),
                diagnostic_provider: Some(DiagnosticProvider::DiagnosticOptions(
                    DiagnosticOptions {
                        identifier: Some(SOURCE.into()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                ..Default::default()
            },
        }
    }
}
