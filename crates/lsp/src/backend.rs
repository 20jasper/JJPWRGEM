use std::sync::OnceLock;

use dashmap::DashMap;
use jjpwrgem_parse::{
    ast::Document,
    format::{JsonMode, format_document},
};
use tower_lsp_server::{Client, LanguageServer, jsonrpc::Result, ls_types::*};

use crate::{
    backend::diagnostics::{build_code_actions, diagnostics_from_error},
    range::{FULL_DOCUMENT_RANGE, PositionEncoding, position_to_byte_offset},
};

const SOURCE: &str = "jjpwrgem";

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: DashMap<Uri, jjpwrgem_parse::Result<Document<String>>>,
    position_encoding: OnceLock<PositionEncoding>,
    client_pulls_diagnostics: OnceLock<bool>,
}

mod diagnostics;

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
    encoding: PositionEncoding,
) {
    let Some(range) = change.range else {
        *text = change.text;
        return;
    };

    let start = position_to_byte_offset(text, range.start, encoding);
    let end = position_to_byte_offset(text, range.end, encoding);
    if start <= end && end <= text.len() {
        text.replace_range(start..end, &change.text);
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            position_encoding: OnceLock::new(),
            client_pulls_diagnostics: OnceLock::new(),
        }
    }

    fn position_encoding(&self) -> PositionEncoding {
        self.position_encoding.get().copied().unwrap_or_default()
    }

    fn client_pulls_diagnostics(&self) -> bool {
        self.client_pulls_diagnostics
            .get()
            .copied()
            .unwrap_or_default()
    }

    async fn process_document(&self, uri: Uri, text: String) {
        let document = Document::parse(text);
        let diagnostics = (!self.client_pulls_diagnostics())
            .then(|| diagnostics_for(&uri, &document, self.position_encoding()));
        self.documents.insert(uri.clone(), document);
        if let Some(diagnostics) = diagnostics {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }
}

#[expect(clippy::unused_async_trait_impl, reason = "trait requires async fn")]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let client_supports_utf8 = params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_ref())
            .is_some_and(|encodings| encodings.contains(&PositionEncodingKind::UTF8));

        let encoding = if client_supports_utf8 {
            PositionEncoding::Utf8
        } else {
            PositionEncoding::Utf16
        };
        let _ = self.position_encoding.set(encoding);
        let _ = self
            .client_pulls_diagnostics
            .set(client_supports_pull_diagnostics(&params.capabilities));

        let position_encoding = client_supports_utf8.then_some(PositionEncodingKind::UTF8);

        Ok(InitializeResult {
            // deprecated clangd extension superseded by positionEncoding in LSP 3.17
            // https://clangd.llvm.org/extensions.html#utf-8-offsets
            offset_encoding: None,
            server_info: Some(ServerInfo {
                name: SOURCE.into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                position_encoding,
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some(SOURCE.into()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                ..Default::default()
            },
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.process_document(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let Some((_, document)) = self.documents.remove(&uri) else {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("received change for unopened document: {uri:?}"),
                )
                .await;
            return;
        };

        let mut text = match document {
            Ok(document) => document.into_source(),
            Err(error) => error.into_source_text(),
        };
        for change in params.content_changes {
            apply_change(&mut text, change, self.position_encoding());
        }

        self.process_document(uri, text).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        if !self.client_pulls_diagnostics() {
            self.client.publish_diagnostics(uri, vec![], None).await;
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = &params.text_document.uri;
        let items = self.documents.get(uri).map_or_else(Vec::new, |document| {
            diagnostics_for(uri, &document, self.position_encoding())
        });
        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items,
                },
            }),
        ))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let Some(document) = self.documents.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let Ok(document) = &*document else {
            return Ok(None);
        };

        let formatted = format_document(document, &JsonMode::default());
        Ok(Some(vec![TextEdit {
            range: FULL_DOCUMENT_RANGE,
            new_text: formatted,
        }]))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let Some(document) = self.documents.get(uri) else {
            return Ok(None);
        };
        Ok(Some(build_code_actions(
            uri,
            &document,
            self.position_encoding(),
        )))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
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
        Uri::from_file_path("/test.json").expect("valid file path")
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
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
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

    fn change(range: Option<Range>, text: &str) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangeEvent {
            range,
            range_length: None,
            text: text.into(),
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
        apply_change(&mut text, change(None, "true"), PositionEncoding::Utf16);
        assert_eq!(text, "true");
    }

    #[test]
    fn applies_incremental_text_change() {
        let mut text = r#"{"a":1}"#.to_owned();
        apply_change(
            &mut text,
            change(Some(range((0, 5), (0, 6))), "2"),
            PositionEncoding::Utf16,
        );
        assert_eq!(text, r#"{"a":2}"#);
    }

    #[test]
    fn applies_incremental_change_with_utf16_surrogate_pair() {
        let mut text = "\"😀\"".to_owned();
        apply_change(
            &mut text,
            change(Some(range((0, 1), (0, 3))), "x"),
            PositionEncoding::Utf16,
        );
        assert_eq!(text, "\"x\"");
    }
}
