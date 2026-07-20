// Pull-only diagnostics server: the first textDocument/diagnostic request
// answers a Full report with one diagnostic, every request after that
// answers an empty Full report -- for exercising wait_for_diagnostics_cleared's
// pull-mode re-poll loop.
use std::cell::Cell;

use lsp_server::{Connection, Message, Response};
use lspresso::{
    gen_lsp_types::{
        Diagnostic, DiagnosticSeverity, DocumentDiagnosticReport, DocumentDiagnosticRequest,
        FullDocumentDiagnosticReport, Position, Range, RelatedFullDocumentDiagnosticReport,
        Request as _,
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

    let requests = Cell::new(0u32);
    helpers::run_event_loop(&conn, |msg, conn| {
        let Message::Request(req) = &msg else {
            return;
        };
        if req.method != DocumentDiagnosticRequest::METHOD.as_str() {
            return;
        }
        let count = requests.get() + 1;
        requests.set(count);
        let report: DocumentDiagnosticReport = if count == 1 {
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some("r1".to_owned()),
                    items: vec![error_diag()],
                },
            }
        } else {
            RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some("r2".to_owned()),
                    items: vec![],
                },
            }
        }
        .into();
        conn.sender
            .send(Message::Response(Response::new_ok(
                req.id.clone(),
                serde_json::to_value(report).unwrap(),
            )))
            .unwrap();
    });
}
