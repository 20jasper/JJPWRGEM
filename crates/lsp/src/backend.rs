use std::sync::OnceLock;

use dashmap::DashMap;
use jjpwrgem_parse::{
    ast::Document,
    file_kind::{JsonKind, kind_from_str},
    format::{LineEnding, prettify_document},
    jsonlines::JsonlinesDocument,
};
use tower_lsp_server::{Client, LanguageServer, jsonrpc::Result, ls_types::*};

use crate::{
    backend::diagnostics::{build_code_actions, diagnostics_from_error},
    range::{FULL_DOCUMENT_RANGE, PositionEncoding},
};

const SOURCE: &str = "jjpwrgem";

#[derive(Debug)]
pub(crate) enum ParsedDocument {
    Json(jjpwrgem_parse::Result<Document<String>>),
    JsonLines(jjpwrgem_parse::Result<JsonlinesDocument<String>>),
}

#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: DashMap<Uri, ParsedDocument>,
    position_encoding: OnceLock<PositionEncoding>,
}

mod diagnostics;

fn identify_file_kind(uri: &Uri, language_id: &str) -> JsonKind {
    uri.to_file_path()
        .and_then(|p| p.extension().and_then(|e| e.to_str().and_then(kind_from_str)))
        .or_else(|| kind_from_str(language_id))
        .unwrap_or(JsonKind::Json)
}

fn analyze_document(
    uri: &Uri,
    kind: JsonKind,
    text: String,
    encoding: PositionEncoding,
) -> (ParsedDocument, Vec<Diagnostic>) {
    match kind {
        JsonKind::Json => match Document::parse(text) {
            Ok(doc) => (ParsedDocument::Json(Ok(doc)), vec![]),
            Err(e) => {
                let diagnostics = diagnostics_from_error(uri, e.source_text(), &e, encoding);
                (ParsedDocument::Json(Err(e)), diagnostics)
            }
        },
        JsonKind::JsonLines => match JsonlinesDocument::parse(text) {
            Ok(doc) => (ParsedDocument::JsonLines(Ok(doc)), vec![]),
            Err(e) => {
                let diagnostics = diagnostics_from_error(uri, e.source_text(), &e, encoding);
                (ParsedDocument::JsonLines(Err(e)), diagnostics)
            }
        },
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

    async fn process_document(&self, uri: Uri, kind: JsonKind, text: String) {
        let (document, diagnostics) = analyze_document(&uri, kind, text, self.position_encoding());
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
        let kind = identify_file_kind(
            &params.text_document.uri,
            &params.text_document.language_id,
        );
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "file kind: {kind:?} (language_id={:?})",
                    params.text_document.language_id
                ),
            )
            .await;
        self.process_document(params.text_document.uri, kind, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let [change] = &*params.content_changes else {
            self.client
                .log_message(MessageType::WARNING, "expected exactly one content change")
                .await;
            return;
        };
        let kind = identify_file_kind(&params.text_document.uri, "");
        self.process_document(params.text_document.uri, kind, change.text.clone())
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
        let text = match &*document {
            ParsedDocument::Json(Ok(doc)) => prettify_document(doc, 80, LineEnding::Lf),
            ParsedDocument::JsonLines(Ok(jl_doc)) => {
                match jjpwrgem_parse::jsonlines::format(jl_doc.source()) {
                    Ok(s) => s,
                    Err(_) => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(vec![TextEdit {
            range: FULL_DOCUMENT_RANGE,
            new_text: text,
        }]))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let Some(document) = self.documents.get(uri) else {
            return Ok(None);
        };
        Ok(Some(build_code_actions(
            uri,
            &*document,
            self.position_encoding(),
        )))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    fn json_uri() -> Uri {
        Uri::from_file_path("/test.json").expect("valid file path")
    }

    fn jsonl_uri() -> Uri {
        Uri::from_file_path("/test.jsonl").expect("valid file path")
    }

    #[test]
    fn invalid_json_produces_error_diagnostic() {
        let (_, diagnostics) = analyze_document(
            &json_uri(),
            JsonKind::Json,
            r#"{"a": }"#.to_owned(),
            PositionEncoding::Utf16,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[0].source.as_deref(), Some(SOURCE));
    }

    #[test]
    fn trailing_comma_produces_code_action() {
        let uri = json_uri();
        let (document, _) = analyze_document(
            &uri,
            JsonKind::Json,
            r#"{"a": 1,}"#.to_owned(),
            PositionEncoding::Utf16,
        );
        let actions = build_code_actions(&uri, &document, PositionEncoding::Utf16);
        assert!(!actions.is_empty());
    }

    #[test]
    fn invalid_jsonlines_produces_error_diagnostic() {
        let (_, diagnostics) = analyze_document(
            &jsonl_uri(),
            JsonKind::JsonLines,
            r#"{"a":"#.to_owned(),
            PositionEncoding::Utf16,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[0].source.as_deref(), Some(SOURCE));
    }

    #[test]
    fn language_id_used_when_no_extension() {
        let uri = Uri::from_str("file:///test").unwrap();
        assert_eq!(identify_file_kind(&uri, "jsonl"), JsonKind::JsonLines);
    }

    #[test]
    fn extension_wins_over_language_id() {
        let uri = Uri::from_str("file:///test.jsonl").unwrap();
        assert_eq!(identify_file_kind(&uri, "json"), JsonKind::JsonLines);
    }
}
