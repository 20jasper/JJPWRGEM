use std::{
    io,
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
};

use gen_lsp_types::*;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::value::RawValue;

use crate::{
    backend::{Backend, Lifecycle},
    lsp_server::req_queue::Id,
    transport::{Rejection, Sink},
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

impl<S: Sink> Backend<S> {
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
            Err(message) => self.out.fail(failed_id, ErrorCodes::InternalError, message),
        }
    }

    fn dispatch_request(
        &mut self,
        id: Id,
        method: &str,
        params: Option<&RawValue>,
    ) -> io::Result<()> {
        let method = LspRequestMethod::from(method);
        match self.lifecycle {
            Lifecycle::Uninitialized => match method {
                LspRequestMethod::Initialize => match parse_params::<InitializeParams>(params) {
                    Ok(params) => {
                        let result = self.initialize(params);
                        self.out.respond::<InitializeRequest>(id, result)
                    }
                    Err(error) => self.invalid_params(id, &error),
                },
                _ => self.out.fail(
                    id,
                    ErrorCodes::ServerNotInitialized,
                    "server not initialized",
                ),
            },
            Lifecycle::ShutDown => {
                self.out
                    .fail(id, ErrorCodes::InvalidRequest, "server is shut down")
            }
            Lifecycle::Running => match method {
                LspRequestMethod::Shutdown => {
                    self.lifecycle = Lifecycle::ShutDown;
                    self.out.respond::<ShutdownRequest>(id, ())
                }
                LspRequestMethod::TextDocumentDiagnostic => {
                    self.respond::<DocumentDiagnosticRequest>(id, params, Self::diagnostic)
                }
                LspRequestMethod::TextDocumentFormatting => {
                    self.respond::<DocumentFormattingRequest>(id, params, Self::formatting)
                }
                LspRequestMethod::TextDocumentCodeAction => {
                    self.respond::<CodeActionRequest>(id, params, Self::code_action)
                }
                LspRequestMethod::TextDocumentDocumentSymbol => {
                    self.respond::<DocumentSymbolRequest>(id, params, Self::document_symbol)
                }
                LspRequestMethod::TextDocumentFoldingRange => {
                    self.respond::<FoldingRangeRequest>(id, params, Self::folding_range)
                }
                LspRequestMethod::TextDocumentDocumentColor => {
                    self.respond::<DocumentColorRequest>(id, params, Self::document_color)
                }
                LspRequestMethod::TextDocumentColorPresentation => self
                    .respond::<ColorPresentationRequest>(id, params, |_, params| {
                        super::colors::color_presentation(params)
                    }),
                LspRequestMethod::TextDocumentSelectionRange => {
                    self.respond::<SelectionRangeRequest>(id, params, Self::selection_range)
                }
                method => self.out.fail(
                    id,
                    ErrorCodes::MethodNotFound,
                    format!("method not found: {method}"),
                ),
            },
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
        if self.lifecycle != Lifecycle::Running {
            return;
        }
        match LspNotificationMethod::from(method) {
            LspNotificationMethod::Initialized => self.initialized(),
            LspNotificationMethod::TextDocumentDidOpen => {
                self.notify(params, Self::did_open);
            }
            LspNotificationMethod::TextDocumentDidChange => {
                self.notify(params, Self::did_change);
            }
            LspNotificationMethod::TextDocumentDidClose => {
                self.notify(params, Self::did_close);
            }
            LspNotificationMethod::WorkspaceDidChangeConfiguration => {
                self.notify(params, Self::did_change_configuration);
            }
            _ => {}
        }
    }

    fn respond<R: Request>(
        &mut self,
        id: Id,
        params: Option<&RawValue>,
        handler: impl FnOnce(&mut Self, &R::Params) -> R::Result,
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
            ErrorCodes::InvalidParams,
            format!("invalid params: {error}"),
        )
    }

    fn notify<P: DeserializeOwned>(
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
