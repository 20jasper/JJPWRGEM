use std::collections::HashMap;

use gen_lsp_types::*;
use jjpwrgem_parse::{
    ast::Document,
    diagnostics::{Context, Patch},
};

use crate::{
    backend::{Backend, SOURCE},
    range::{PositionEncoding, span_to_range},
    transport::Sink,
};

fn diagnostics_from_error(
    uri: &Uri,
    text: &str,
    error: &jjpwrgem_parse::Error,
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    let related: Vec<_> = Vec::<Context<'_>>::from(error)
        .into_iter()
        .map(|ctx| DiagnosticRelatedInformation {
            location: Location {
                uri: uri.clone(),
                range: span_to_range(text, ctx.span, encoding),
            },
            message: ctx.message.into_owned(),
        })
        .collect();

    vec![Diagnostic {
        range: span_to_range(text, *error.range(), encoding),
        severity: Some(DiagnosticSeverity::Error),
        source: Some(SOURCE.into()),
        message: error.message().into(),
        related_information: (!related.is_empty()).then_some(related),
        ..Default::default()
    }]
}

fn build_code_actions(
    uri: &Uri,
    document: &jjpwrgem_parse::Result<Document<String>>,
    encoding: PositionEncoding,
) -> Vec<CodeActionResponse> {
    let Err(error) = document else {
        return vec![];
    };
    let text = error.source_text();
    let diagnostics = Some(diagnostics_from_error(uri, text, error, encoding));
    Vec::<Patch<'_>>::from(error)
        .into_iter()
        .map(|patch| {
            let changes = HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: span_to_range(text, patch.span, encoding),
                    new_text: patch.replacement.into_owned(),
                }],
            )]);
            CodeActionResponse::CodeAction(CodeAction {
                title: patch.message.into_owned(),
                kind: Some(CodeActionKind::QuickFix),
                diagnostics: diagnostics.clone(),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
        .collect()
}

pub(super) fn diagnostics_for(
    uri: &Uri,
    document: &jjpwrgem_parse::Result<Document<String>>,
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    match document {
        Ok(_) => vec![],
        Err(error) => diagnostics_from_error(uri, error.source_text(), error, encoding),
    }
}

impl<S: Sink> Backend<S> {
    pub(super) fn diagnostic(
        &mut self,
        params: &DocumentDiagnosticParams,
    ) -> DocumentDiagnosticReport {
        let uri = &params.text_document.uri;
        let items = self.documents.get(uri).map_or_else(Vec::new, |open| {
            diagnostics_for(uri, &open.parsed, self.position_encoding)
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

    pub(super) fn code_action(
        &mut self,
        params: &CodeActionParams,
    ) -> Option<Vec<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let open = self.documents.get(uri)?;
        Some(build_code_actions(
            uri,
            &open.parsed,
            self.position_encoding,
        ))
    }
}

#[cfg(test)]
mod tests {
    use jjpwrgem_parse::SyntaxOptions;

    use super::*;

    fn dummy_uri() -> Uri {
        Uri::parse("file:///test.json".to_owned()).expect("valid uri")
    }

    fn parse(text: &str) -> jjpwrgem_parse::Result<Document<String>> {
        Document::parse(text.to_owned(), SyntaxOptions::JSON)
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
}
