use std::sync::OnceLock;

use dashmap::DashMap;
use jjpwrgem_parse::{
    ast::Document,
    format::{LineEnding, prettify_document},
};
use tower_lsp_server::{Client, LanguageServer, jsonrpc::Result, ls_types::*};

use crate::{
    backend::diagnostics::{build_code_actions, diagnostics_from_error},
    range::{FULL_DOCUMENT_RANGE, PositionEncoding},
};

const SOURCE: &str = "jjpwrgem";

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: DashMap<Uri, jjpwrgem_parse::Result<Document<String>>>,
    position_encoding: OnceLock<PositionEncoding>,
}

mod diagnostics;

fn analyze_document(
    uri: &Uri,
    text: String,
    encoding: PositionEncoding,
) -> (jjpwrgem_parse::Result<Document<String>>, Vec<Diagnostic>) {
    match Document::parse(text) {
        Ok(doc) => (Ok(doc), vec![]),
        Err(e) => {
            let diagnostics = diagnostics_from_error(uri, e.source_text(), &e, encoding);
            (Err(e), diagnostics)
        }
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            position_encoding: OnceLock::new(),
        }
    }

    fn position_encoding(&self) -> PositionEncoding {
        self.position_encoding.get().copied().unwrap_or_default()
    }

    async fn process_document(&self, uri: Uri, text: String) {
        let (document, diagnostics) = analyze_document(&uri, text, self.position_encoding());
        self.documents.insert(uri.clone(), document);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

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
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.process_document(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let [change] = &*params.content_changes else {
            self.client
                .log_message(MessageType::WARNING, "expected exactly one content change")
                .await;
            return;
        };
        self.process_document(params.text_document.uri, change.text.clone())
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let Some(document) = self.documents.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let Ok(document) = &*document else {
            return Ok(None);
        };

        let formatted = prettify_document(document, 80, LineEnding::Lf);
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

#[cfg(test)]
mod tests {

    use super::*;

    fn dummy_uri() -> Uri {
        Uri::from_file_path("/test.json").expect("valid file path")
    }

    #[test]
    fn valid_json_produces_no_diagnostics() {
        let (_, diagnostics) = analyze_document(
            &dummy_uri(),
            r#"{"a": 1}"#.to_owned(),
            PositionEncoding::Utf16,
        );
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn invalid_json_produces_error_diagnostic() {
        let (_, diagnostics) = analyze_document(
            &dummy_uri(),
            r#"{"a": }"#.to_owned(),
            PositionEncoding::Utf16,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[0].source.as_deref(), Some(SOURCE));
    }

    #[test]
    fn trailing_comma_produces_code_action() {
        let uri = dummy_uri();
        let (document, _) =
            analyze_document(&uri, r#"{"a": 1,}"#.to_owned(), PositionEncoding::Utf16);
        let actions = build_code_actions(&uri, &document, PositionEncoding::Utf16);
        assert!(!actions.is_empty());
    }

    #[test]
    fn valid_json_produces_no_code_actions() {
        let uri = dummy_uri();
        let (document, _) =
            analyze_document(&uri, r#"{"a": 1}"#.to_owned(), PositionEncoding::Utf16);
        let actions = build_code_actions(&uri, &document, PositionEncoding::Utf16);
        assert!(actions.is_empty());
    }
}
