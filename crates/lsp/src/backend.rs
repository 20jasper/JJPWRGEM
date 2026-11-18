use std::{
    any::Any,
    collections::HashMap,
    io,
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
    process::ExitCode,
};

use gen_lsp_types::*;
use jjpwrgem_parse::{
    ast::Document,
    format::{FormatSettings, JsonMode, format_document},
};
use serde::Deserialize;
use serde_json::{Value, value::RawValue};

use crate::{
    backend::{
        diagnostics::{build_code_actions, diagnostics_from_error},
        folding::folding_ranges,
        settings::format_settings,
        symbols::document_symbols,
    },
    lsp_server::{
        msg::ErrorCode,
        req_queue::{Id, Outgoing},
    },
    range::{FULL_DOCUMENT_RANGE, PositionEncoding, Positions},
    transport::{Rejection, Writer},
};

const SOURCE: &str = "jjpwrgem";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Uninitialized,
    Running,
    ShutDown,
}

pub struct Backend {
    out: Writer,
    outgoing: Outgoing<()>,
    lifecycle: Lifecycle,
    documents: HashMap<Uri, jjpwrgem_parse::Result<Document<String>>>,
    position_encoding: PositionEncoding,
    client_pulls_diagnostics: bool,
    format: FormatSettings,
}

mod diagnostics;
mod folding;
mod settings;
mod symbols;

fn diagnostics_for(
    uri: &Uri,
    document: &jjpwrgem_parse::Result<Document<String>>,
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    match document {
        Ok(_) => vec![],
        Err(e) => diagnostics_from_error(uri, e.source_text(), e, encoding),
    }
}

fn apply_change(
    text: &mut String,
    change: TextDocumentContentChangeEvent,
    positions: &Positions,
) -> bool {
    let (range, new_text) = match change {
        TextDocumentContentChangeEvent::TextDocumentContentChangePartial(change) => {
            (change.range, change.text)
        }
        TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(change) => {
            *text = change.text;
            return true;
        }
    };

    let start = positions.offset(text, range.start);
    let end = positions.offset(text, range.end);
    if start > end || end > text.len() {
        return false;
    }

    text.replace_range(start..end, &new_text);
    true
}

fn parse_params<'a, P: Deserialize<'a>>(
    params: Option<&'a RawValue>,
) -> Result<P, serde_json::Error> {
    serde_json::from_str(params.map_or("null", RawValue::get))
}

impl Backend {
    pub fn new(out: Writer) -> Self {
        Self {
            out,
            outgoing: Outgoing::new(),
            lifecycle: Lifecycle::Uninitialized,
            documents: HashMap::new(),
            position_encoding: PositionEncoding::Utf16,
            client_pulls_diagnostics: false,
            format: FormatSettings::default(),
        }
    }

    pub fn exit_code(&self) -> ExitCode {
        if self.lifecycle == Lifecycle::ShutDown {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }

    fn initialize(&mut self, params: InitializeParams) -> InitializeResult {
        let client_supports_utf8 = params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_ref())
            .is_some_and(|encodings| encodings.contains(&PositionEncodingKind::UTF8));

        self.lifecycle = Lifecycle::Running;
        self.position_encoding = if client_supports_utf8 {
            PositionEncoding::Utf8
        } else {
            PositionEncoding::Utf16
        };
        self.client_pulls_diagnostics = client_supports_pull_diagnostics(&params.capabilities);
        if let Some(options) = params.initialization_options {
            self.update_format_settings(options);
        }

        let position_encoding = client_supports_utf8.then_some(PositionEncodingKind::UTF8);
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

    pub fn response(&mut self, id: &Id) {
        self.outgoing.complete(id);
    }

    pub fn reject(&mut self, rejection: Rejection) -> io::Result<()> {
        self.out
            .fail(rejection.id, rejection.code, rejection.message)
    }

    pub fn request(&mut self, id: Id, method: &str, params: Option<&RawValue>) -> io::Result<()> {
        let failed_id = id.clone();
        match catch_unwind(AssertUnwindSafe(|| {
            self.dispatch_request(id, method, params)
        })) {
            Ok(result) => result,
            Err(panic) => self.out.fail(
                failed_id,
                ErrorCode::InternalError,
                format!("internal error: {}", panic_message(&*panic)),
            ),
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
                "server not initialized".to_owned(),
            ),
            (Lifecycle::ShutDown, _) => self.out.fail(
                id,
                ErrorCode::InvalidRequest,
                "server is shut down".to_owned(),
            ),
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
        if let Err(panic) = catch_unwind(AssertUnwindSafe(|| {
            self.dispatch_notification(method, params);
        })) {
            self.error(format!("internal error: {}", panic_message(&*panic)));
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

    fn error(&mut self, message: impl Into<String>) {
        self.log_message(MessageType::Error, message);
    }

    fn warn(&mut self, message: impl Into<String>) {
        self.log_message(MessageType::Warning, message);
    }

    fn log_message(&mut self, kind: MessageType, message: impl Into<String>) {
        self.out.notify::<LogMessageNotification>(LogMessageParams {
            kind,
            message: message.into(),
        });
    }

    fn with_document<T>(&self, uri: &Uri, f: impl FnOnce(&Document<String>) -> T) -> Option<T> {
        self.documents.get(uri)?.as_ref().ok().map(f)
    }

    fn update_format_settings(&mut self, value: Value) {
        self.format = match format_settings(value) {
            Ok(settings) => settings,
            Err(error) => {
                self.out
                    .notify::<ShowMessageNotification>(ShowMessageParams {
                        kind: MessageType::Warning,
                        message: format!(
                            "{SOURCE}: invalid format settings, using defaults: {error}"
                        ),
                    });
                FormatSettings::default()
            }
        };
    }

    fn process_document(&mut self, uri: Uri, text: String) {
        let document = Document::parse(text);
        let diagnostics = (!self.client_pulls_diagnostics)
            .then(|| diagnostics_for(&uri, &document, self.position_encoding));
        self.documents.insert(uri.clone(), document);
        if let Some(diagnostics) = diagnostics {
            self.publish_diagnostics(uri, diagnostics);
        }
    }

    fn publish_diagnostics(&mut self, uri: Uri, diagnostics: Vec<Diagnostic>) {
        self.out
            .notify::<PublishDiagnosticsNotification>(PublishDiagnosticsParams {
                uri,
                version: None,
                diagnostics,
            });
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        self.process_document(params.text_document.uri, params.text_document.text);
    }

    fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        self.update_format_settings(params.settings);
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.text_document_identifier.uri;
        let Some(document) = self.documents.remove(&uri) else {
            self.warn(format!("received change for unopened document: {uri:?}"));
            return;
        };

        let mut text = match document {
            Ok(document) => document.into_source(),
            Err(error) => error.into_source_text(),
        };
        for change in params.content_changes {
            let positions = Positions::new(&text, self.position_encoding);
            if !apply_change(&mut text, change, &positions) {
                self.warn("ignored invalid content change");
            }
        }

        self.process_document(uri, text);
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        if !self.client_pulls_diagnostics {
            self.publish_diagnostics(uri, vec![]);
        }
    }

    fn diagnostic(&self, params: &DocumentDiagnosticParams) -> DocumentDiagnosticReport {
        let uri = &params.text_document.uri;
        let items = self.documents.get(uri).map_or_else(Vec::new, |document| {
            diagnostics_for(uri, document, self.position_encoding)
        });
        DocumentDiagnosticReport::RelatedFullDocumentDiagnosticReport(
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items,
                },
            },
        )
    }

    fn formatting(&self, params: &DocumentFormattingParams) -> Option<Vec<TextEdit>> {
        let mode = JsonMode::from_settings(&self.format);
        self.with_document(&params.text_document.uri, |document| {
            vec![TextEdit {
                range: FULL_DOCUMENT_RANGE,
                new_text: format_document(document, &mode),
            }]
        })
    }

    fn code_action(&self, params: &CodeActionParams) -> Option<Vec<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let document = self.documents.get(uri)?;
        Some(build_code_actions(uri, document, self.position_encoding))
    }

    fn document_symbol(&self, params: &DocumentSymbolParams) -> Option<DocumentSymbolResponse> {
        self.with_document(&params.text_document.uri, |document| {
            DocumentSymbolResponse::DocumentSymbolList(document_symbols(
                document,
                self.position_encoding,
            ))
        })
    }

    fn folding_range(&self, params: &FoldingRangeParams) -> Option<Vec<FoldingRange>> {
        self.with_document(&params.text_document.uri, |document| {
            folding_ranges(document, self.position_encoding)
        })
    }
}

fn panic_message(panic: &dyn Any) -> &str {
    panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("panic")
}

fn client_supports_pull_diagnostics(capabilities: &ClientCapabilities) -> bool {
    capabilities
        .text_document
        .as_ref()
        .is_some_and(|t| t.diagnostic.is_some())
}

#[cfg(test)]
mod tests {

    use super::*;

    fn dummy_uri() -> Uri {
        Uri::parse("file:///test.json".to_owned()).expect("valid uri")
    }

    fn parse(text: &str) -> jjpwrgem_parse::Result<Document<String>> {
        Document::parse(text.to_owned())
    }

    #[test]
    fn valid_json_produces_no_diagnostics() {
        let diagnostics =
            diagnostics_for(&dummy_uri(), &parse(r#"{"a": 1}"#), PositionEncoding::Utf16);
        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn invalid_json_produces_error_diagnostic() {
        let diagnostics =
            diagnostics_for(&dummy_uri(), &parse(r#"{"a": }"#), PositionEncoding::Utf16);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::Error));
        assert_eq!(diagnostics[0].source.as_deref(), Some(SOURCE));
    }

    #[test]
    fn trailing_comma_produces_code_action() {
        let actions = build_code_actions(
            &dummy_uri(),
            &parse(r#"{"a": 1,}"#),
            PositionEncoding::Utf16,
        );
        assert_ne!(actions, Vec::new());
    }

    #[test]
    fn valid_json_produces_no_code_actions() {
        let actions =
            build_code_actions(&dummy_uri(), &parse(r#"{"a": 1}"#), PositionEncoding::Utf16);
        assert_eq!(actions, Vec::new());
    }

    #[expect(deprecated, reason = "the field is required to build the struct")]
    fn change(range: Option<Range>, text: &str) -> TextDocumentContentChangeEvent {
        match range {
            Some(range) => TextDocumentContentChangePartial {
                range,
                range_length: None,
                text: text.into(),
            }
            .into(),
            None => TextDocumentContentChangeWholeDocument { text: text.into() }.into(),
        }
    }

    fn range(start: (u32, u32), end: (u32, u32)) -> Range {
        Range {
            start: Position {
                line: start.0,
                character: start.1,
            },
            end: Position {
                line: end.0,
                character: end.1,
            },
        }
    }

    #[test]
    fn applies_full_text_change() {
        let mut text = r#"{"a":1}"#.to_owned();
        let positions = Positions::new(&text, PositionEncoding::Utf16);
        apply_change(&mut text, change(None, "true"), &positions);
        assert_eq!(text, "true");
    }

    #[test]
    fn applies_incremental_text_change() {
        let mut text = r#"{"a":1}"#.to_owned();
        let positions = Positions::new(&text, PositionEncoding::Utf16);
        apply_change(
            &mut text,
            change(Some(range((0, 5), (0, 6))), "2"),
            &positions,
        );
        assert_eq!(text, r#"{"a":2}"#);
    }

    #[test]
    fn applies_incremental_change_with_utf16_surrogate_pair() {
        let mut text = "\"😀\"".to_owned();
        let positions = Positions::new(&text, PositionEncoding::Utf16);
        apply_change(
            &mut text,
            change(Some(range((0, 1), (0, 3))), "x"),
            &positions,
        );
        assert_eq!(text, "\"x\"");
    }

    #[test]
    fn applies_insertion_at_end() {
        let mut text = r#"{"a":1}"#.to_owned();
        let positions = Positions::new(&text, PositionEncoding::Utf16);
        assert!(apply_change(
            &mut text,
            change(Some(range((0, 7), (0, 7))), ","),
            &positions,
        ));
        assert_eq!(text, r#"{"a":1},"#);
    }

    #[test]
    fn ignores_reversed_range() {
        let mut text = r#"{"a":1}"#.to_owned();
        let positions = Positions::new(&text, PositionEncoding::Utf16);
        assert!(!apply_change(
            &mut text,
            change(Some(range((0, 5), (0, 2))), "x"),
            &positions,
        ));
        assert_eq!(text, r#"{"a":1}"#);
    }
}
