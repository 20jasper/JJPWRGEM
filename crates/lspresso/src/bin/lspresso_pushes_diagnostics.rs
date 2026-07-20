// Push-only diagnostics server (no diagnosticProvider capability, matching
// jjpwrgem-lsp). On didOpen it publishes one diagnostic for the opened uri;
// on didChange it publishes an empty batch for the same uri, simulating a
// fix.
use lsp_server::{Connection, Message, Notification as RpcNotification};
use lspresso::{
    gen_lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentNotification,
        DidChangeTextDocumentParams, DidOpenTextDocumentNotification, DidOpenTextDocumentParams,
        Notification as _, Position, PublishDiagnosticsNotification, PublishDiagnosticsParams,
        Range, Uri,
    },
    test_server as helpers,
};

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

fn publish(conn: &Connection, uri: Uri, diagnostics: Vec<Diagnostic>) {
    let params = PublishDiagnosticsParams {
        uri,
        version: None,
        diagnostics,
    };
    conn.sender
        .send(Message::Notification(RpcNotification {
            method: PublishDiagnosticsNotification::METHOD.as_str().to_owned(),
            params: serde_json::to_value(params).unwrap(),
        }))
        .unwrap();
}

fn main() {
    let (conn, _io) = Connection::stdio();
    helpers::handle_initialize(&conn);

    helpers::run_event_loop(&conn, |msg, conn| {
        let Message::Notification(note) = &msg else {
            return;
        };
        if note.method == DidOpenTextDocumentNotification::METHOD.as_str() {
            let params: DidOpenTextDocumentParams =
                serde_json::from_value(note.params.clone()).unwrap();
            publish(conn, params.text_document.uri, vec![error_diag()]);
        } else if note.method == DidChangeTextDocumentNotification::METHOD.as_str() {
            let params: DidChangeTextDocumentParams =
                serde_json::from_value(note.params.clone()).unwrap();
            publish(
                conn,
                params.text_document.text_document_identifier.uri,
                vec![],
            );
        }
    });
}
