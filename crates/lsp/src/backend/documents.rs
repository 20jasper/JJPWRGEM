use gen_lsp_types::*;
use jjpwrgem_parse::{ast::Document, format::FormatSettings};
use serde_json::Value;

use crate::{
    backend::{Backend, diagnostics::diagnostics_for, settings::format_settings},
    range::Positions,
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

impl Backend {
    pub(super) fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        self.process_document(params.text_document.uri, params.text_document.text);
    }

    pub(super) fn did_change(&mut self, params: DidChangeTextDocumentParams) {
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
                self.show_warning(format!("invalid format settings, using defaults: {error}"));
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
