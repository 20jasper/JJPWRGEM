use gen_lsp_types::*;
use jjpwrgem_parse::{SyntaxOptions, ast::Document, format::FormatSettings};
use serde_json::Value;

use crate::{
    backend::{Backend, OpenDocument, diagnostics::diagnostics_for, settings::format_settings},
    range::Positions,
    transport::Sink,
};

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

impl<S: Sink> Backend<S> {
    pub(super) fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem {
            uri,
            language_id,
            text,
            ..
        } = params.text_document;
        let syntax = if language_id.as_str() == "jsonc" {
            SyntaxOptions::JSONC
        } else {
            SyntaxOptions::JSON
        };
        let parsed = self.parse_document(&uri, text, syntax);
        self.documents.insert(
            uri,
            OpenDocument {
                parsed,
                syntax,
                warned_formatting: false,
            },
        );
    }

    pub(super) fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.text_document_identifier.uri;
        let Some(mut open) = self.documents.remove(&uri) else {
            self.warn(format!("received change for unopened document: {uri:?}"));
            return;
        };

        let mut text = match open.parsed {
            Ok(document) => document.into_source(),
            Err(error) => error.into_source_text(),
        };
        for change in params.content_changes {
            let positions = Positions::new(&text, self.position_encoding);
            if !apply_change(&mut text, change, &positions) {
                self.warn("ignored invalid content change");
            }
        }

        open.parsed = self.parse_document(&uri, text, open.syntax);
        self.documents.insert(uri, open);
    }

    pub(super) fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
        if !self.client_pulls_diagnostics {
            self.publish_diagnostics(uri, vec![]);
        }
    }

    pub(super) fn did_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        self.update_format_settings(params.settings);
    }

    pub(super) fn update_format_settings(&mut self, value: Value) {
        self.format = match format_settings(value) {
            Ok(settings) => settings,
            Err(error) => {
                self.show_warning(&format!("invalid format settings, using defaults: {error}"));
                FormatSettings::default()
            }
        };
    }

    fn parse_document(
        &mut self,
        uri: &Uri,
        text: String,
        syntax: SyntaxOptions,
    ) -> jjpwrgem_parse::Result<Document<String>> {
        let parsed = Document::parse(text, syntax);
        if !self.client_pulls_diagnostics {
            let diagnostics = diagnostics_for(uri, &parsed, self.position_encoding);
            self.publish_diagnostics(uri.clone(), diagnostics);
        }
        parsed
    }

    fn publish_diagnostics(&mut self, uri: Uri, diagnostics: Vec<Diagnostic>) {
        self.out
            .notify::<PublishDiagnosticsNotification>(PublishDiagnosticsParams {
                uri,
                version: None,
                diagnostics,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::PositionEncoding;

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

    fn apply(text: &str, change: TextDocumentContentChangeEvent) -> (bool, String) {
        let mut text = text.to_owned();
        let positions = Positions::new(&text, PositionEncoding::Utf16);
        let applied = apply_change(&mut text, change, &positions);
        (applied, text)
    }

    #[test]
    fn applies_full_text_change() {
        let (_, text) = apply(r#"{"a":1}"#, change(None, "true"));
        assert_eq!(text, "true");
    }

    #[test]
    fn applies_incremental_text_change() {
        let range = Range::new(Position::new(0, 5), Position::new(0, 6));
        let (_, text) = apply(r#"{"a":1}"#, change(Some(range), "2"));
        assert_eq!(text, r#"{"a":2}"#);
    }

    #[test]
    fn applies_incremental_change_with_utf16_surrogate_pair() {
        let range = Range::new(Position::new(0, 1), Position::new(0, 3));
        let (_, text) = apply("\"😀\"", change(Some(range), "x"));
        assert_eq!(text, "\"x\"");
    }

    #[test]
    fn applies_insertion_at_end() {
        let range = Range::new(Position::new(0, 7), Position::new(0, 7));
        let (applied, text) = apply(r#"{"a":1}"#, change(Some(range), ","));
        assert!(applied);
        assert_eq!(text, r#"{"a":1},"#);
    }

    #[test]
    fn ignores_reversed_range() {
        let range = Range::new(Position::new(0, 5), Position::new(0, 2));
        let (applied, text) = apply(r#"{"a":1}"#, change(Some(range), "x"));
        assert!(!applied);
        assert_eq!(text, r#"{"a":1}"#);
    }
}
