use std::collections::HashMap;

use jjpwrgem_parse::{
    ast::Document,
    diagnostics::{Context, Patch},
};
use tower_lsp_server::ls_types::*;

use crate::{
    backend::SOURCE,
    range::{PositionEncoding, span_to_range},
};

pub fn diagnostics_from_error(
    uri: &Uri,
    text: &str,
    e: &jjpwrgem_parse::Error,
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    let related: Vec<_> = Vec::<Context<'_>>::from(e)
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
        range: span_to_range(text, *e.range(), encoding),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(SOURCE.into()),
        message: e.message(),
        related_information: (!related.is_empty()).then_some(related),
        ..Default::default()
    }]
}

pub fn build_code_actions(
    uri: &Uri,
    document: &jjpwrgem_parse::Result<Document<String>>,
    encoding: PositionEncoding,
) -> CodeActionResponse {
    let Err(err) = document else {
        return vec![];
    };
    let text = err.source_text();
    let diagnostics = Some(diagnostics_from_error(uri, text, err, encoding));
    Vec::<Patch<'_>>::from(err)
        .into_iter()
        .map(|patch| {
            let changes = HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: span_to_range(text, patch.span, encoding),
                    new_text: patch.replacement.into_owned(),
                }],
            )]);
            CodeActionOrCommand::CodeAction(CodeAction {
                title: patch.message.into_owned(),
                kind: Some(CodeActionKind::QUICKFIX),
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
