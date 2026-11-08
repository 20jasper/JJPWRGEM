use std::time::Duration;

use gen_lsp_types::*;
use lspresso::{Client, ErrorKind};

use crate::{
    common::*,
    script::{Publications, Step},
};

fn full_report(message: &str, result_id: &str) -> Step {
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
    step::respond_with(report)
}

fn unchanged_report(result_id: &str) -> Step {
    let report: DocumentDiagnosticReport = RelatedUnchangedDocumentDiagnosticReport {
        unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
            result_id: result_id.to_owned(),
        },
        ..Default::default()
    }
    .into();
    step::respond_with(report)
}

fn pull_server(pulls: impl IntoIterator<Item = Step>) -> ScriptedServer {
    ScriptedServer::new()
        .capabilities(ServerCapabilities {
            diagnostic_provider: Some(DiagnosticOptions::default().into()),
            ..Default::default()
        })
        .steps(pulls)
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| match &diagnostic.message {
            Message::String(message) => message.clone(),
            other @ Message::MarkupContent(_) => panic!("expected a plain message, got {other:?}"),
        })
        .collect()
}

fn published(client: &mut Client, uri: &Uri) -> Vec<String> {
    messages(&client.next_diagnostics(uri, TIMEOUT).unwrap())
}

fn pulled(client: &mut Client, uri: &Uri) -> Vec<String> {
    messages(&client.diagnostics(uri).unwrap())
}

fn previous_ids_sent(recording: &Recording) -> Vec<Option<String>> {
    recording
        .requests::<DocumentDiagnosticRequest>()
        .into_iter()
        .map(|pull| pull.previous_result_id)
        .collect()
}

#[test]
fn diagnostics_published_for_an_older_version_are_skipped() {
    let (mut client, uri) = opened();
    assert_eq!(published(&mut client, &uri), ["v1"]);

    client.did_change_full(&uri, "{ }").unwrap();
    assert_eq!(published(&mut client, &uri), ["v2"]);
}

#[test]
fn waiting_on_one_document_keeps_the_diagnostics_of_another() {
    let (mut client, a) = opened();
    let b: Uri = "file:///b.json".parse().unwrap();
    open(&mut client, &b);

    assert_eq!(published(&mut client, &b), ["v1"]);
    assert_eq!(published(&mut client, &a), ["v1"]);
}

#[test]
fn notifications_received_during_another_wait_are_kept() {
    let (mut client, uri) = opened();

    workspace_symbol(&mut client).unwrap();

    assert_eq!(published(&mut client, &uri), ["v1"]);
}

#[test]
fn diagnostics_time_out_when_nothing_is_published() {
    let (mut client, uri) = ScriptedServer::new()
        .respond_to_all()
        .publications(Publications::Off)
        .spawn_opened();

    assert_err_kind!(
        client.next_diagnostics(&uri, Duration::from_millis(100)),
        ErrorKind::NotificationTimeout { .. }
    );
}

#[test]
fn diagnostics_before_initialize_is_rejected() {
    let mut client = uninitialized();
    let uri = a_json();

    assert_not_initialized([
        client.next_diagnostics(&uri, TIMEOUT).unwrap_err(),
        client.diagnostics(&uri).unwrap_err(),
    ]);
}

#[test]
fn a_closed_document_cannot_be_waited_on() {
    let (mut client, uri) = opened();
    client.did_close(&uri).unwrap();

    assert_err_kind!(
        client.next_diagnostics(&uri, TIMEOUT),
        ErrorKind::NotOpen { .. }
    );
}

#[test]
fn an_unversioned_publication_from_before_a_change_is_not_the_answer() {
    let (mut client, uri) = ScriptedServer::new()
        .respond_to_all()
        .publications(Publications::Unversioned)
        .spawn_opened();
    client
        .formatting(&uri, FormattingOptions::default())
        .unwrap();

    client.did_change_full(&uri, "{ }").unwrap();

    assert_eq!(published(&mut client, &uri), ["v2"]);
}

#[test]
fn pushed_diagnostics_are_used_when_pull_is_not_supported() {
    let (mut client, uri) = opened();
    assert!(!client.supports_pull_diagnostics());

    assert_eq!(pulled(&mut client, &uri), ["v1"]);
}

#[test]
fn pull_diagnostics_are_used_when_supported() {
    let (mut client, uri) = pull_server([full_report("pulled", "r1")]).spawn_opened();
    assert!(client.supports_pull_diagnostics());

    assert_eq!(pulled(&mut client, &uri), ["pulled"]);
}

#[test]
fn pull_diagnostics_thread_the_previous_result_id() {
    let recording = Recording::new();
    let (mut client, uri) = pull_server([full_report("pulled", "r1"), unchanged_report("r2")])
        .recording_to(&recording)
        .spawn_opened();

    assert_eq!(pulled(&mut client, &uri), ["pulled"]);
    assert_eq!(pulled(&mut client, &uri), ["pulled"]);

    assert_eq!(previous_ids_sent(&recording), [None, Some("r1".to_owned())]);
}

#[test]
fn reopening_a_document_forgets_its_previous_result_id() {
    let recording = Recording::new();
    let (mut client, uri) = pull_server([full_report("first", "r1"), full_report("second", "r2")])
        .recording_to(&recording)
        .spawn_opened();
    assert_eq!(pulled(&mut client, &uri), ["first"]);

    client.did_close(&uri).unwrap();
    open(&mut client, &uri);
    assert_eq!(pulled(&mut client, &uri), ["second"]);

    assert_eq!(previous_ids_sent(&recording), [None, None]);
}

#[test]
fn a_pull_gives_up_after_repeated_retriable_failures() {
    let recording = Recording::new();
    let (mut client, uri) = pull_server([
        step::fail(LspErrorCodes::ContentModified, ()),
        step::fail(LspErrorCodes::ContentModified, ()),
        step::fail(LspErrorCodes::ContentModified, ()),
        full_report("too late", "r1"),
    ])
    .recording_to(&recording)
    .spawn_opened();

    assert_err_kind!(
        client.diagnostics(&uri),
        ErrorKind::Response { code, .. } if *code == i32::from(LspErrorCodes::ContentModified)
    );
    assert_eq!(previous_ids_sent(&recording), [None, None, None]);
}
