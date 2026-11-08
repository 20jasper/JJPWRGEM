use std::time::Duration;

use gen_lsp_types::*;
use lspresso::{Client, ErrorKind};
use serde_json::json;

use crate::common::*;

fn pull_error(code: LspErrorCodes, retrigger_request: Option<bool>) -> serde_json::Value {
    let data = retrigger_request
        .map(|retrigger_request| DiagnosticServerCancellationData { retrigger_request });
    json!({
        "action": "respond:0",
        "error": { "code": i32::from(code), "message": "no", "data": data }
    })
}

fn pulled(message: &str, result_id: &str) -> serde_json::Value {
    let report: DocumentDiagnosticReport = RelatedFullDocumentDiagnosticReport {
        full_document_diagnostic_report: FullDocumentDiagnosticReport::new(
            Some(result_id.to_owned()),
            vec![Diagnostic {
                message: message.into(),
                ..Default::default()
            }],
        ),
        ..Default::default()
    }
    .into();
    json!({ "action": "respond:0", "result": report })
}

fn unchanged(result_id: &str) -> serde_json::Value {
    let report: DocumentDiagnosticReport = RelatedUnchangedDocumentDiagnosticReport {
        unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
            result_id: result_id.to_owned(),
        },
        ..Default::default()
    }
    .into();
    json!({ "action": "respond:0", "result": report })
}

fn pull_server(pulls: &[serde_json::Value]) -> ScriptedServer {
    let initialize = json!({ "action": "respond:0" });
    let steps: Vec<_> = std::iter::once(&initialize).chain(pulls).collect();
    ScriptedServer::new()
        .capabilities(&ServerCapabilities {
            diagnostic_provider: Some(DiagnosticOptions::default().into()),
            ..Default::default()
        })
        .steps(&json!(steps))
}

fn first_message(diagnostics: &[Diagnostic]) -> &str {
    match &diagnostics[0].message {
        Message::String(message) => message,
        other @ Message::MarkupContent(_) => panic!("expected a plain message, got {other:?}"),
    }
}

fn assert_published(client: &mut Client, uri: &Uri, message: &str) {
    let diagnostics = client.next_diagnostics(uri, TIMEOUT).unwrap();
    assert_eq!(
        first_message(&diagnostics),
        message,
        "diagnostics for {uri:?}"
    );
}

fn assert_pulled(client: &mut Client, uri: &Uri, message: &str) {
    let diagnostics = client.diagnostics(uri).unwrap();
    assert_eq!(
        first_message(&diagnostics),
        message,
        "diagnostics for {uri:?}"
    );
}

#[test]
fn diagnostics_published_for_an_older_version_are_skipped() {
    let mut client = initialized();
    let uri = a_json();

    open(&mut client, &uri);
    assert_published(&mut client, &uri, "v1");

    client.did_change_full(&uri, "{ }").unwrap();
    assert_published(&mut client, &uri, "v2");
}

#[test]
fn waiting_on_one_document_keeps_the_diagnostics_of_another() {
    let mut client = initialized();
    let a = a_json();
    let b: Uri = "file:///b.json".parse().unwrap();
    open(&mut client, &a);
    open(&mut client, &b);

    assert_published(&mut client, &b, "v1");
    assert_published(&mut client, &a, "v1");
}

#[test]
fn notifications_received_during_another_wait_are_kept() {
    let mut client = initialized();
    let uri = a_json();
    open(&mut client, &uri);

    workspace_symbol(&mut client).unwrap();

    assert_published(&mut client, &uri, "v1");
}

#[test]
fn diagnostics_time_out_when_nothing_is_published() {
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .without_diagnostics()
        .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);

    let err = client
        .next_diagnostics(&uri, Duration::from_millis(100))
        .unwrap_err();
    assert_kind!(err, ErrorKind::NotificationTimeout { .. });
}

#[test]
fn diagnostics_before_initialize_is_rejected() {
    let mut client = scripted(&json!([]));
    let uri = a_json();
    for err in [
        client.next_diagnostics(&uri, TIMEOUT).unwrap_err(),
        client.diagnostics(&uri).unwrap_err(),
    ] {
        assert_kind!(err, ErrorKind::NotInitialized);
    }
}

#[test]
fn a_closed_document_cannot_be_waited_on() {
    let mut client = initialized();
    let uri = a_json();
    open(&mut client, &uri);
    client.did_close(&uri).unwrap();

    let err = client.next_diagnostics(&uri, TIMEOUT).unwrap_err();
    assert_kind!(err, ErrorKind::NotOpen { .. });
}

#[test]
fn an_unversioned_publication_from_before_a_change_is_not_the_answer() {
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .unversioned()
        .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);
    client
        .formatting(&uri, FormattingOptions::default())
        .unwrap();

    client.did_change_full(&uri, "{ }").unwrap();

    assert_published(&mut client, &uri, "v2");
}

#[test]
fn pushed_diagnostics_are_used_when_pull_is_not_supported() {
    let mut client = initialized();
    assert!(!client.supports_pull_diagnostics());

    let uri = a_json();
    open(&mut client, &uri);
    assert_pulled(&mut client, &uri, "v1");
}

#[test]
fn pull_diagnostics_are_used_when_supported() {
    let mut client = pull_server(&[pulled("pulled", "r1")]).spawn_initialized();
    assert!(client.supports_pull_diagnostics());

    let uri = a_json();
    open(&mut client, &uri);
    assert_pulled(&mut client, &uri, "pulled");
}

#[test]
fn pull_diagnostics_thread_the_previous_result_id() {
    let recording = Recording::new();
    let mut client = pull_server(&[pulled("pulled", "r1"), unchanged("r2")])
        .recording_to(&recording)
        .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);

    assert_pulled(&mut client, &uri, "pulled");
    assert_pulled(&mut client, &uri, "pulled");

    let pulls = recording.sent("textDocument/diagnostic");
    assert_eq!(pulls.len(), 2);
    assert!(pulls[0]["params"].get("previousResultId").is_none());
    assert_eq!(pulls[1]["params"]["previousResultId"], "r1");
}

#[test]
fn reopening_a_document_forgets_its_previous_result_id() {
    let recording = Recording::new();
    let mut client = pull_server(&[pulled("first", "r1"), pulled("second", "r2")])
        .recording_to(&recording)
        .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);
    assert_pulled(&mut client, &uri, "first");

    client.did_close(&uri).unwrap();
    open(&mut client, &uri);
    assert_pulled(&mut client, &uri, "second");

    let pulls = recording.sent("textDocument/diagnostic");
    assert!(pulls[1]["params"].get("previousResultId").is_none());
}

#[test]
fn a_pull_is_sent_again_when_the_content_was_modified() {
    let mut client = pull_server(&[
        pull_error(LspErrorCodes::ContentModified, None),
        pull_error(LspErrorCodes::ServerCancelled, Some(true)),
        pulled("pulled", "r1"),
    ])
    .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);

    assert_pulled(&mut client, &uri, "pulled");
}

#[test]
fn a_pull_gives_up_after_repeated_retriable_failures() {
    let recording = Recording::new();
    let mut client = pull_server(&[
        pull_error(LspErrorCodes::ContentModified, None),
        pull_error(LspErrorCodes::ContentModified, None),
        pull_error(LspErrorCodes::ContentModified, None),
        pulled("too late", "r1"),
    ])
    .recording_to(&recording)
    .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);

    let err = client.diagnostics(&uri).unwrap_err();
    assert_kind!(err, ErrorKind::Response { code, .. } if *code == i32::from(LspErrorCodes::ContentModified));
    assert_eq!(recording.sent("textDocument/diagnostic").len(), 3);
}

#[test]
fn a_pull_is_not_retried_after_other_failures() {
    let recording = Recording::new();
    let mut client = pull_server(&[
        pull_error(LspErrorCodes::RequestFailed, None),
        pulled("unused", "r1"),
    ])
    .recording_to(&recording)
    .spawn_initialized();
    let uri = a_json();
    open(&mut client, &uri);

    let err = client.diagnostics(&uri).unwrap_err();
    assert_kind!(err, ErrorKind::Response { code, .. } if *code == i32::from(LspErrorCodes::RequestFailed));
    assert_eq!(recording.sent("textDocument/diagnostic").len(), 1);
}
