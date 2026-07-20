// Pull-only diagnostics server (declares diagnosticProvider). Answers
// textDocument/diagnostic with a Full report carrying one diagnostic and
// resultId "r1" -- unless the request's previousResultId is already "r1",
// in which case it answers Unchanged (same resultId), exercising both
// resultid-threading and the client's Unchanged-falls-back-to-cache path.
use lsp_server::{Connection, Message, Response};
use lspresso::{
    gen_lsp_types::{
        Diagnostic, DiagnosticSeverity, DocumentDiagnosticParams, DocumentDiagnosticReport,
        DocumentDiagnosticRequest, FullDocumentDiagnosticReport, Position, Range,
        RelatedFullDocumentDiagnosticReport, RelatedUnchangedDocumentDiagnosticReport,
        Request as _, UnchangedDocumentDiagnosticReport,
    },
    test_server as helpers,
};
use serde_json::json;

fn error_diag() -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position::new(0, 0),
            end: Position::new(0, 1),
        },
        severity: Some(DiagnosticSeverity::Error),
        message: "boom".into(),
        ..Default::default()
    }
}

fn main() {
    let (conn, _io) = Connection::stdio();
    helpers::handle_initialize_with(
        &conn,
        json!({"capabilities": {"diagnosticProvider": {
            "interFileDependencies": false,
            "workspaceDiagnostics": false,
        }}}),
    );

    helpers::run_event_loop(&conn, |msg, conn| {
        let Message::Request(req) = &msg else {
            return;
        };
        if req.method != DocumentDiagnosticRequest::METHOD.as_str() {
            return;
        }
        let params: DocumentDiagnosticParams = serde_json::from_value(req.params.clone()).unwrap();
        let report: DocumentDiagnosticReport = if params.previous_result_id.as_deref() == Some("r1")
        {
            RelatedUnchangedDocumentDiagnosticReport {
                related_documents: None,
                unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
                    result_id: "r1".to_owned(),
                },
            }
            .into()
        } else {
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some("r1".to_owned()),
                    items: vec![error_diag()],
                },
            }
            .into()
        };
        conn.sender
            .send(Message::Response(Response::new_ok(
                req.id.clone(),
                serde_json::to_value(report).unwrap(),
            )))
            .unwrap();
    });
}
