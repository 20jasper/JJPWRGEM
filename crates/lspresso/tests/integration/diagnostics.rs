use std::{process::Command, time::Duration};

use gen_lsp_types::*;
use lspresso::{Client, ErrorKind};
use serde_json::json;

use crate::common::*;

fn scripted_without_diagnostics(steps: &serde_json::Value) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env("LSPRESSO_NO_DIAGNOSTICS", "1");
    let mut client = Client::spawn(command).expect("the scripted server should spawn");
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

fn unversioned_scripted(steps: &serde_json::Value) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env("LSPRESSO_UNVERSIONED", "1");
    let mut client = Client::spawn(command).expect("the scripted server should spawn");
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

fn pull_error(code: LspErrorCodes, retrigger_request: Option<bool>) -> serde_json::Value {
    let data = retrigger_request
        .map(|retrigger_request| DiagnosticServerCancellationData { retrigger_request });
    json!({
        "action": "respond:0",
        "error": { "code": i32::from(code), "message": "no", "data": data }
    })
}

fn pull_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        diagnostic_provider: Some(DiagnosticOptions::default().into()),
        ..Default::default()
    }
}

fn diagnostic_report(message: &str, result_id: &str) -> DocumentDiagnosticReport {
    RelatedFullDocumentDiagnosticReport {
        full_document_diagnostic_report: FullDocumentDiagnosticReport::new(
            Some(result_id.to_owned()),
            vec![Diagnostic {
                message: message.into(),
                ..Default::default()
            }],
        ),
        ..Default::default()
    }
    .into()
}

fn unchanged_report(result_id: &str) -> DocumentDiagnosticReport {
    RelatedUnchangedDocumentDiagnosticReport {
        unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
            result_id: result_id.to_owned(),
        },
        ..Default::default()
    }
    .into()
}

fn first_message(diagnostics: &[Diagnostic]) -> &str {
    match &diagnostics[0].message {
        Message::String(message) => message,
        other @ Message::MarkupContent(_) => panic!("expected a plain message, got {other:?}"),
    }
}

#[test]
fn diagnostics_published_for_an_older_version_are_skipped() {
    let mut client = scripted_initialized(&json!([{ "action": "respond:0" }]));
    let uri = a_json();

    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    assert_eq!(
        first_message(&client.next_diagnostics(&uri, TIMEOUT).unwrap()),
        "v1"
    );

    client.did_change_full(&uri, "{ }").unwrap();
    assert_eq!(
        first_message(&client.next_diagnostics(&uri, TIMEOUT).unwrap()),
        "v2"
    );
}

#[test]
fn waiting_on_one_document_keeps_another_document_s_diagnostics() {
    let mut client = spawn_scripted(1);
    let a = a_json();
    let b: Uri = "file:///b.json".parse().unwrap();
    client
        .did_open(a.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    client
        .did_open(b.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    assert_eq!(
        first_message(&client.next_diagnostics(&b, TIMEOUT).unwrap()),
        "v1"
    );
    assert_eq!(
        first_message(&client.next_diagnostics(&a, TIMEOUT).unwrap()),
        "v1"
    );
}

#[test]
fn notifications_received_during_another_wait_are_kept() {
    let mut client = spawn_scripted(2);
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    workspace_symbol(&mut client).unwrap();

    assert_eq!(
        first_message(&client.next_diagnostics(&uri, TIMEOUT).unwrap()),
        "v1"
    );
}

#[test]
fn diagnostics_time_out_when_nothing_is_published() {
    let mut client = scripted_without_diagnostics(&json!([{ "action": "respond:0" }]));
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    let err = client
        .next_diagnostics(&uri, Duration::from_millis(100))
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::DiagnosticsTimeout { .. }),
        "{err:?}"
    );
}

#[test]
fn diagnostics_before_initialize_is_rejected() {
    let mut client = scripted(&json!([]));
    let uri = a_json();
    for err in [
        client.next_diagnostics(&uri, TIMEOUT).unwrap_err(),
        client.diagnostics(&uri).unwrap_err(),
    ] {
        assert!(matches!(err.kind(), ErrorKind::NotInitialized), "{err:?}");
    }
}

#[test]
fn a_closed_document_cannot_be_waited_on() {
    let mut client = spawn_scripted(1);
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    client.did_close(&uri).unwrap();

    let err = client.next_diagnostics(&uri, TIMEOUT).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");
}

#[test]
fn pull_diagnostics_are_used_when_supported() {
    let mut client = initialized_with_capabilities_and_script(
        &pull_capabilities(),
        &json!([
            { "action": "respond:0" },
            { "action": "respond:0", "result": diagnostic_report("pulled", "r1") }
        ]),
    );
    assert!(client.supports_pull_diagnostics());

    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    let diagnostics = client.diagnostics(&uri).unwrap();
    assert_eq!(first_message(&diagnostics), "pulled");
}

#[test]
fn pushed_diagnostics_are_used_when_pull_is_not_supported() {
    let mut client = scripted_initialized(&json!([{ "action": "respond:0" }]));
    assert!(!client.supports_pull_diagnostics());

    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    let diagnostics = client.diagnostics(&uri).unwrap();
    assert_eq!(first_message(&diagnostics), "v1");
}

#[test]
fn pull_diagnostics_thread_the_previous_result_id() {
    let record = std::env::temp_dir().join(format!(
        "lspresso-diagnostics-{}-{}.jsonl",
        std::process::id(),
        line!()
    ));
    if record.exists() {
        std::fs::remove_file(&record).expect("removing the stale recording");
    }

    let mut client = initialized_with_capabilities_and_recording(
        &pull_capabilities(),
        &json!([
            { "action": "respond:0" },
            { "action": "respond:0", "result": diagnostic_report("pulled", "r1") },
            { "action": "respond:0", "result": unchanged_report("r2") }
        ]),
        &record,
    );
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    assert_eq!(first_message(&client.diagnostics(&uri).unwrap()), "pulled");
    assert_eq!(first_message(&client.diagnostics(&uri).unwrap()), "pulled");

    let messages: Vec<serde_json::Value> = std::fs::read_to_string(&record)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    std::fs::remove_file(&record).expect("removing the recording");

    let pulls: Vec<_> = messages
        .iter()
        .filter(|message| message["method"] == "textDocument/diagnostic")
        .collect();
    assert_eq!(pulls.len(), 2);
    assert!(pulls[0]["params"].get("previousResultId").is_none());
    assert_eq!(pulls[1]["params"]["previousResultId"], "r1");
}

#[test]
fn an_unversioned_publication_from_before_a_change_is_not_the_answer() {
    let mut client =
        unversioned_scripted(&json!([{ "action": "respond:0" }, { "action": "respond:0" }]));
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    client
        .formatting(&uri, FormattingOptions::default())
        .unwrap();

    client.did_change_full(&uri, "{ }").unwrap();

    assert_eq!(
        first_message(&client.next_diagnostics(&uri, TIMEOUT).unwrap()),
        "v2"
    );
}

#[test]
fn a_pull_is_sent_again_when_the_content_was_modified() {
    let mut client = initialized_with_capabilities_and_script(
        &pull_capabilities(),
        &json!([
            { "action": "respond:0" },
            pull_error(LspErrorCodes::ContentModified, None),
            pull_error(LspErrorCodes::ServerCancelled, Some(true)),
            { "action": "respond:0", "result": diagnostic_report("pulled", "r1") }
        ]),
    );
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    assert_eq!(first_message(&client.diagnostics(&uri).unwrap()), "pulled");
}

#[test]
fn a_pull_gives_up_after_repeated_retriable_failures() {
    let mut client = initialized_with_capabilities_and_script(
        &pull_capabilities(),
        &json!([
            { "action": "respond:0" },
            pull_error(LspErrorCodes::ContentModified, None),
            pull_error(LspErrorCodes::ContentModified, None),
            pull_error(LspErrorCodes::ContentModified, None),
            { "action": "respond:0", "result": diagnostic_report("too late", "r1") }
        ]),
    );
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    let err = client.diagnostics(&uri).unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::Response { code, .. } if *code == i32::from(LspErrorCodes::ContentModified)),
        "{err}"
    );
}

#[test]
fn a_pull_is_not_retried_after_other_failures() {
    let mut client = initialized_with_capabilities_and_script(
        &pull_capabilities(),
        &json!([
            { "action": "respond:0" },
            pull_error(LspErrorCodes::RequestFailed, None),
            { "action": "respond:0", "result": diagnostic_report("unused", "r1") }
        ]),
    );
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    let err = client.diagnostics(&uri).unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::Response { code, .. } if *code == i32::from(LspErrorCodes::RequestFailed)),
        "{err}"
    );
}
