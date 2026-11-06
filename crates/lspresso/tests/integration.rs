use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use gen_lsp_types::*;
use lspresso::{Client, ErrorKind, Protocol};
use serde_json::json;

const TIMEOUT: Duration = Duration::from_secs(5);

/// Spawns the scripted server; each step is how it answers one request.
fn scripted(steps: &serde_json::Value) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    Client::spawn(command).expect("the scripted server should spawn")
}

fn scripted_initialized(steps: &serde_json::Value) -> Client {
    let mut client = scripted(steps);
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

fn scripted_recording(steps: &serde_json::Value, record: &Path) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env("LSPRESSO_RECORD", record);
    Client::spawn(command).expect("the scripted server should spawn")
}

fn initialized_with_capabilities(capabilities: &serde_json::Value) -> Client {
    initialized_with_capabilities_and_script(capabilities, &json!([{ "action": "respond:0" }]))
}

fn initialized_with_capabilities_and_script(
    capabilities: &serde_json::Value,
    steps: &serde_json::Value,
) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env(
        "LSPRESSO_INITIALIZE_RESULT",
        json!({ "capabilities": capabilities }).to_string(),
    );
    let mut client = Client::spawn(command).expect("the scripted server should spawn");
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

fn initialized_with_capabilities_and_recording(
    capabilities: &serde_json::Value,
    steps: &serde_json::Value,
    record: &Path,
) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env(
        "LSPRESSO_INITIALIZE_RESULT",
        json!({ "capabilities": capabilities }).to_string(),
    );
    command.env("LSPRESSO_RECORD", record);
    let mut client = Client::spawn(command).expect("the scripted server should spawn");
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

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

fn pull_error(code: i32, data: &serde_json::Value) -> serde_json::Value {
    json!({ "action": "respond:0", "error": { "code": code, "message": "no", "data": data } })
}

const CONTENT_MODIFIED: i32 = -32801;
const SERVER_CANCELLED: i32 = -32802;
const REQUEST_FAILED: i32 = -32803;

fn pull_capabilities() -> serde_json::Value {
    json!({
        "diagnosticProvider": {
            "interFileDependencies": false,
            "workspaceDiagnostics": false
        }
    })
}

fn diagnostic_report(message: &str, result_id: &str) -> serde_json::Value {
    json!({
        "kind": "full",
        "items": [{
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 0 }
            },
            "message": message
        }],
        "resultId": result_id
    })
}

fn first_message(diagnostics: &[Diagnostic]) -> serde_json::Value {
    serde_json::to_value(diagnostics).unwrap()[0]["message"].clone()
}

fn a_json() -> Uri {
    "file:///a.json".parse().unwrap()
}

fn notification<'a>(messages: &'a [serde_json::Value], method: &str) -> &'a serde_json::Value {
    messages
        .iter()
        .find(|message| message["method"] == method)
        .unwrap_or_else(|| panic!("missing {method} notification"))
}

fn jjp_dist_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("JJP_BIN") {
        return PathBuf::from(path);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/dist/jjp")
}

fn spawn_real_server() -> Client {
    let bin = jjp_dist_binary_path();
    assert!(
        bin.exists(),
        "jjp binary not found at {}; run `cargo build --profile dist -p jjpwrgem` first, \
         or set JJP_BIN to an explicit path",
        bin.display()
    );
    let mut command = Command::new(bin);
    command.arg("lsp");
    Client::spawn(command).expect("jjp lsp should spawn")
}

fn workspace_symbol(client: &mut Client) -> lspresso::Result<()> {
    let symbols = client.request::<gen_lsp_types::WorkspaceSymbolRequest>(
        gen_lsp_types::WorkspaceSymbolParams {
            query: String::new(),
            ..Default::default()
        },
    )?;
    assert_eq!(symbols, None, "the scripted server answers null");
    Ok(())
}

#[test]
fn initialize_and_shutdown_round_trip() {
    let mut client = scripted(&json!([{ "action": "respond:0" }, { "action": "respond:0" }]));

    let result = client.initialize(ClientCapabilities::default()).unwrap();
    assert_eq!(
        result.capabilities,
        gen_lsp_types::ServerCapabilities::default()
    );

    client.shutdown().unwrap();

    let status = client.wait_for_exit(TIMEOUT).unwrap();
    assert!(status.success());
}

#[test]
fn exits_immediately_is_reported_as_disconnected() {
    let mut client = scripted(&json!([{ "action": "exit" }]));

    let err = client
        .initialize(ClientCapabilities::default())
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::Disconnected { .. }));

    let retry_err = client
        .initialize(ClientCapabilities::default())
        .unwrap_err();
    assert!(!matches!(retry_err.kind(), ErrorKind::AlreadyInitialized));
}

#[test]
fn never_responds_times_out() {
    let mut client = scripted(&json!([]));
    client.set_request_timeout(Duration::from_millis(100));

    let err = client
        .initialize(ClientCapabilities::default())
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));
}

#[test]
fn wait_for_exit_times_out_instead_of_blocking_forever_on_a_hung_process() {
    let mut client = scripted(&json!([]));

    let err = client
        .wait_for_exit(Duration::from_millis(100))
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ExitTimeout { .. }));
}

#[test]
fn spawning_a_missing_program_names_it_in_the_error() {
    let err = Client::spawn(Command::new("lspresso-no-such-server")).unwrap_err();

    assert!(
        matches!(err.kind(), ErrorKind::Spawn { program, .. } if program == "lspresso-no-such-server")
    );
    assert!(err.to_string().contains("lspresso-no-such-server"));
}

#[test]
#[cfg(unix)]
fn wait_for_exit_times_out_when_stdout_closes_but_the_process_keeps_running() {
    let mut command = Command::new("sh");
    command.args(["-c", "exec >&-; sleep 30"]);
    let mut client = Client::spawn(command).unwrap();

    let err = client
        .wait_for_exit(Duration::from_millis(200))
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ExitTimeout { .. }));
}

fn spawn_scripted(responses: usize) -> Client {
    scripted_initialized(&json!(vec![json!({ "action": "respond:0" }); responses]))
}

#[test]
fn rss_is_reported_only_while_the_server_runs() {
    let mut client = spawn_scripted(2);
    let rss = client.rss_bytes().expect("the server is running");
    assert!(rss > 256 * 1024, "implausibly small RSS: {rss} bytes");

    client.shutdown().unwrap();
    client.wait_for_exit(TIMEOUT).unwrap();
    assert_eq!(client.rss_bytes(), None);
}

#[test]
fn any_request_can_be_sent_through_the_typed_escape_hatch() {
    let mut client = spawn_scripted(2);

    workspace_symbol(&mut client).unwrap();
}

#[test]
fn the_escape_hatch_rejects_calls_before_initialize() {
    let mut client = scripted(&json!([]));

    let err = workspace_symbol(&mut client).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotInitialized), "{err:?}");
    let err = client
        .notify::<gen_lsp_types::DidChangeConfigurationNotification>(
            gen_lsp_types::DidChangeConfigurationParams {
                settings: serde_json::Value::Null,
            },
        )
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotInitialized), "{err:?}");
}

#[test]
fn a_document_opens_once_until_closed() {
    let mut client = spawn_scripted(1);
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    let err = client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::AlreadyOpen { .. }),
        "{err:?}"
    );

    client.did_close(&uri).unwrap();
    let err = client.did_change_full(&uri, "[]").unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");
    client.did_open(uri, "{}", LanguageKind::JSON).unwrap();
}

#[test]
fn raw_document_notifications_are_tracked_too() {
    let mut client = spawn_scripted(1);
    let uri = a_json();
    client
        .protocol_did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: LanguageKind::JSON,
                version: 7,
                text: "{}".to_owned(),
            },
        })
        .unwrap();

    let err = client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::AlreadyOpen { .. }),
        "{err:?}"
    );
    client.did_change_full(&uri, "[]").unwrap();
    client
        .protocol_did_close(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        })
        .unwrap();
    let err = client.did_change_full(&uri, "[]").unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");
}

#[test]
fn save_notifications_need_an_open_document() {
    let mut client = spawn_scripted(2);
    let uri = a_json();

    let errs = [
        client.did_save(&uri, None).unwrap_err(),
        client
            .will_save(&uri, TextDocumentSaveReason::Manual)
            .unwrap_err(),
        client
            .will_save_wait_until(&uri, TextDocumentSaveReason::Manual)
            .unwrap_err(),
    ];
    for err in errs {
        assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");
    }

    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    client.did_save(&uri, None).unwrap();
    client
        .will_save(&uri, TextDocumentSaveReason::Manual)
        .unwrap();
}

#[test]
fn document_sync_before_initialize_is_rejected() {
    let mut client = scripted(&json!([]));
    let uri = a_json();

    let errs = [
        client
            .did_open(uri.clone(), "{}", LanguageKind::JSON)
            .unwrap_err(),
        client.did_change_full(&uri, "[]").unwrap_err(),
        client.did_close(&uri).unwrap_err(),
        client.did_save(&uri, None).unwrap_err(),
        client
            .will_save(&uri, TextDocumentSaveReason::Manual)
            .unwrap_err(),
        client
            .will_save_wait_until(&uri, TextDocumentSaveReason::Manual)
            .unwrap_err(),
    ];
    for err in errs {
        assert!(matches!(err.kind(), ErrorKind::NotInitialized), "{err:?}");
    }
}

#[test]
fn document_sync_sends_versions_and_text() {
    let record = std::env::temp_dir().join(format!(
        "lspresso-record-{}-{}.jsonl",
        std::process::id(),
        line!()
    ));
    if record.exists() {
        std::fs::remove_file(&record).expect("removing the stale recording");
    }

    let mut client = scripted_recording(
        &json!([{ "action": "respond:0" }, { "action": "respond:0" }]),
        &record,
    );
    client.initialize(ClientCapabilities::default()).unwrap();

    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
    client.did_change_full(&uri, "[]").unwrap();
    client.did_change_full(&uri, "[1]").unwrap();
    client.did_close(&uri).unwrap();
    client.shutdown().unwrap();
    client.wait_for_exit(TIMEOUT).unwrap();

    let messages: Vec<serde_json::Value> = std::fs::read_to_string(&record)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    std::fs::remove_file(&record).expect("removing the recording");

    let opened = notification(&messages, "textDocument/didOpen");
    assert_eq!(opened["params"]["textDocument"]["uri"], "file:///a.json");
    assert_eq!(opened["params"]["textDocument"]["version"], 1);
    assert_eq!(opened["params"]["textDocument"]["text"], "{}");
    assert_eq!(opened["params"]["textDocument"]["languageId"], "json");

    let changes: Vec<_> = messages
        .iter()
        .filter(|message| message["method"] == "textDocument/didChange")
        .collect();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0]["params"]["textDocument"]["version"], 2);
    assert_eq!(changes[0]["params"]["contentChanges"][0]["text"], "[]");
    assert_eq!(changes[1]["params"]["textDocument"]["version"], 3);
    assert_eq!(changes[1]["params"]["contentChanges"][0]["text"], "[1]");

    let closed = notification(&messages, "textDocument/didClose");
    assert_eq!(closed["params"]["textDocument"]["uri"], "file:///a.json");
}

#[test]
fn sync_kind_comes_from_the_initialize_result() {
    let incremental = initialized_with_capabilities(&json!({ "textDocumentSync": 2 }));
    assert_eq!(
        incremental.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Incremental)
    );
    assert!(incremental.supports_incremental_sync());

    let full = initialized_with_capabilities(&json!({ "textDocumentSync": { "change": 1 } }));
    assert_eq!(
        full.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Full)
    );
    assert!(!full.supports_incremental_sync());
}

#[test]
fn sync_kind_is_unknown_before_initialize() {
    let client = scripted(&json!([]));
    assert_eq!(client.text_document_sync_kind(), None);
    assert!(!client.supports_incremental_sync());
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
            { "action": "respond:0", "result": { "kind": "unchanged", "resultId": "r2" } }
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
            pull_error(CONTENT_MODIFIED, &json!(null)),
            pull_error(
                SERVER_CANCELLED,
                &json!({ "retriggerRequest": true })
            ),
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
            pull_error(CONTENT_MODIFIED, &json!(null)),
            pull_error(CONTENT_MODIFIED, &json!(null)),
            pull_error(CONTENT_MODIFIED, &json!(null)),
            { "action": "respond:0", "result": diagnostic_report("too late", "r1") }
        ]),
    );
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    let err = client.diagnostics(&uri).unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::Response { code, .. } if *code == CONTENT_MODIFIED),
        "{err}"
    );
}

#[test]
fn a_pull_is_not_retried_after_other_failures() {
    let mut client = initialized_with_capabilities_and_script(
        &pull_capabilities(),
        &json!([
            { "action": "respond:0" },
            pull_error(REQUEST_FAILED, &json!(null)),
            { "action": "respond:0", "result": diagnostic_report("unused", "r1") }
        ]),
    );
    let uri = a_json();
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();

    let err = client.diagnostics(&uri).unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::Response { code, .. } if *code == REQUEST_FAILED),
        "{err}"
    );
}

#[test]
#[ignore = "requires a separately built target/dist/jjp or JJP_BIN"]
fn initialize_against_the_real_server() {
    let mut client = spawn_real_server();

    let result = client.initialize(ClientCapabilities::default()).unwrap();
    assert_eq!(result.server_info.unwrap().name, "jjpwrgem");

    client.shutdown().unwrap();
    let status = client.wait_for_exit(TIMEOUT).unwrap();
    assert!(status.success());
}
