use std::{
    io,
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
};

use gen_lsp_types::*;
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::{
    backend::{Backend, Lifecycle},
    lsp_server::{msg::ErrorCode, req_queue::Id},
    transport::Rejection,
};

fn parse_params<'a, P: Deserialize<'a>>(
    params: Option<&'a RawValue>,
) -> Result<P, serde_json::Error> {
    serde_json::from_str(params.map_or("null", RawValue::get))
}

fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|panic| {
        let message = panic
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("panic");
        format!("internal error: {message}")
    })
}

impl Backend {
    pub fn response(&mut self, id: &Id) {
        self.outgoing.complete(id);
    }

    pub fn reject(&mut self, rejection: Rejection) -> io::Result<()> {
        self.out
            .fail(rejection.id, rejection.code, rejection.message)
    }

    pub fn request(&mut self, id: Id, method: &str, params: Option<&RawValue>) -> io::Result<()> {
        let failed_id = id.clone();
        match guarded(|| self.dispatch_request(id, method, params)) {
            Ok(result) => result,
            Err(message) => self.out.fail(failed_id, ErrorCode::InternalError, message),
        }
    }

    fn dispatch_request(
        &mut self,
        id: Id,
        method: &str,
        params: Option<&RawValue>,
    ) -> io::Result<()> {
        let method = LspRequestMethod::from(method);
        match (self.lifecycle, method) {
            (Lifecycle::Uninitialized, LspRequestMethod::Initialize) => {
                match parse_params::<InitializeParams>(params) {
                    Ok(params) => {
                        let result = self.initialize(params);
                        self.out.respond::<InitializeRequest>(id, result)
                    }
                    Err(error) => self.invalid_params(id, &error),
                }
            }
            (Lifecycle::Uninitialized, _) => self.out.fail(
                id,
                ErrorCode::ServerNotInitialized,
                "server not initialized",
            ),
            (Lifecycle::ShutDown, _) => {
                self.out
                    .fail(id, ErrorCode::InvalidRequest, "server is shut down")
            }
            (Lifecycle::Running, LspRequestMethod::Shutdown) => {
                self.lifecycle = Lifecycle::ShutDown;
                self.out.respond::<ShutdownRequest>(id, ())
            }
            (Lifecycle::Running, LspRequestMethod::TextDocumentDiagnostic) => {
                self.respond::<DocumentDiagnosticRequest>(id, params, Self::diagnostic)
            }
            (Lifecycle::Running, LspRequestMethod::TextDocumentFormatting) => {
                self.respond::<DocumentFormattingRequest>(id, params, Self::formatting)
            }
            (Lifecycle::Running, LspRequestMethod::TextDocumentCodeAction) => {
                self.respond::<CodeActionRequest>(id, params, Self::code_action)
            }
            (Lifecycle::Running, LspRequestMethod::TextDocumentDocumentSymbol) => {
                self.respond::<DocumentSymbolRequest>(id, params, Self::document_symbol)
            }
            (Lifecycle::Running, LspRequestMethod::TextDocumentFoldingRange) => {
                self.respond::<FoldingRangeRequest>(id, params, Self::folding_range)
            }
            (Lifecycle::Running, method) => self.out.fail(
                id,
                ErrorCode::MethodNotFound,
                format!("method not found: {method}"),
            ),
        }
    }

    pub fn notification(&mut self, method: &str, params: Option<&RawValue>) -> ControlFlow<()> {
        if LspNotificationMethod::from(method) == LspNotificationMethod::Exit {
            return ControlFlow::Break(());
        }
        if let Err(message) = guarded(|| self.dispatch_notification(method, params)) {
            self.error(message);
        }
        ControlFlow::Continue(())
    }

    fn dispatch_notification(&mut self, method: &str, params: Option<&RawValue>) {
        match (self.lifecycle, LspNotificationMethod::from(method)) {
            (Lifecycle::Running, LspNotificationMethod::TextDocumentDidOpen) => {
                self.notify(params, Self::did_open);
            }
            (Lifecycle::Running, LspNotificationMethod::TextDocumentDidChange) => {
                self.notify(params, Self::did_change);
            }
            (Lifecycle::Running, LspNotificationMethod::TextDocumentDidClose) => {
                self.notify(params, Self::did_close);
            }
            (Lifecycle::Running, LspNotificationMethod::WorkspaceDidChangeConfiguration) => {
                self.notify(params, Self::did_change_configuration);
            }
            _ => {}
        }
    }

    fn respond<R: Request>(
        &mut self,
        id: Id,
        params: Option<&RawValue>,
        handler: impl FnOnce(&Self, &R::Params) -> R::Result,
    ) -> io::Result<()> {
        match parse_params(params) {
            Ok(params) => {
                let result = handler(self, &params);
                self.out.respond::<R>(id, result)
            }
            Err(error) => self.invalid_params(id, &error),
        }
    }

    fn invalid_params(&mut self, id: Id, error: &serde_json::Error) -> io::Result<()> {
        self.out.fail(
            id,
            ErrorCode::InvalidParams,
            format!("invalid params: {error}"),
        )
    }

    fn notify<P: for<'de> Deserialize<'de>>(
        &mut self,
        params: Option<&RawValue>,
        handler: impl FnOnce(&mut Self, P),
    ) {
        match parse_params(params) {
            Ok(params) => handler(self, params),
            Err(error) => self.warn(format!("invalid notification params: {error}")),
        }
    }
}
