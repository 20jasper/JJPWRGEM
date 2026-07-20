// Pull-only diagnostics server that always answers an empty Full report --
// for the "server has nothing to report" side of the pull path.
use lsp_server::{Connection, Message, Response};
use lspresso::{
    gen_lsp_types::{
        DocumentDiagnosticReport, DocumentDiagnosticRequest, FullDocumentDiagnosticReport,
        RelatedFullDocumentDiagnosticReport, Request as _,
    },
    test_server as helpers,
};
use serde_json::json;

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
        let report: DocumentDiagnosticReport = RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                result_id: Some("e1".to_owned()),
                items: vec![],
            },
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
