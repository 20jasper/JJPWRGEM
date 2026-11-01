use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use lspresso::{Client, ErrorKind};
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
    client.initialize(lspresso::capabilities::vscode()).unwrap();
    client
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

#[test]
fn initialize_and_shutdown_round_trip() {
    let mut client = scripted(&json!([{ "action": "respond:0" }, { "action": "respond:0" }]));

    let result = client.initialize(lspresso::capabilities::vscode()).unwrap();
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
        .initialize(lspresso::capabilities::vscode())
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::Disconnected { .. }));

    let retry_err = client
        .initialize(lspresso::capabilities::vscode())
        .unwrap_err();
    assert!(!matches!(retry_err.kind(), ErrorKind::AlreadyInitialized));
}

#[test]
fn never_responds_times_out() {
    let mut client = scripted(&json!([]));
    client.set_request_timeout(Duration::from_millis(100));

    let err = client
        .initialize(lspresso::capabilities::vscode())
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
fn a_constant_stream_of_notifications_does_not_stop_timeouts_firing() {
    let mut client =
        scripted_initialized(&json!([{ "action": "respond:0" }, { "action": "drip" }]));
    client.set_request_timeout(Duration::from_millis(200));
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    client.next_diagnostics(&uri, TIMEOUT).unwrap();

    let err = client.formatting(&uri).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));

    let err = client
        .next_diagnostics(&uri, Duration::from_millis(200))
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::DiagnosticsTimeout { .. }));
}

#[test]
fn a_malformed_frame_is_reported_as_such_not_as_a_disconnect() {
    let mut client =
        scripted_initialized(&json!([{ "action": "respond:0" }, { "action": "garbage" }]));

    let err = client.shutdown().unwrap_err();
    assert!(matches!(
        err.kind(),
        ErrorKind::MalformedMessage {
            method: "shutdown",
            ..
        }
    ));
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
fn diagnostics_published_for_an_older_version_are_skipped() {
    let mut client = scripted_initialized(&json!([{ "action": "respond:0" }]));
    let uri = lspresso::parse_uri("file:///a.json");

    // The scripted server publishes a stale diagnostic before each real one.
    let message = |client: &mut Client| {
        let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();
        serde_json::to_value(&diagnostics).unwrap()[0]["message"].clone()
    };

    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    assert_eq!(message(&mut client), "v1");

    client.did_change_full(&uri, "{ }").unwrap();
    assert_eq!(message(&mut client), "v2");
}

#[test]
fn server_requests_are_answered_so_a_server_waiting_on_one_does_not_stall() {
    let mut client = scripted_initialized(&json!([
        { "server_request": true, "action": "respond:0" },
        { "action": "respond:0" },
    ]));

    client.shutdown().unwrap();
    assert!(client.wait_for_exit(TIMEOUT).unwrap().success());
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

#[test]
fn a_server_that_stops_reading_stdin_times_out_instead_of_blocking_the_caller() {
    let mut client =
        scripted_initialized(&json!([{ "action": "respond:0" }, { "action": "stall" }]));
    client.set_request_timeout(Duration::from_millis(100));
    let uri = lspresso::parse_uri("file:///big.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    let err = client.formatting(&uri).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));

    // Many pipe buffers' worth: a write on the caller's thread would block forever.
    client
        .did_change_full(&uri, "x".repeat(4 * 1024 * 1024))
        .unwrap();

    let err = client.formatting(&uri).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));
}

fn spawn_scripted(responses: usize) -> Client {
    scripted_initialized(&json!(vec![json!({ "action": "respond:0" }); responses]))
}

#[test]
fn code_action_sends_the_given_diagnostics() {
    let mut client = spawn_scripted(2);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();
    let range = lspresso::Range::new(lspresso::Position::new(0, 0), lspresso::Position::new(0, 0));

    let actions = client
        .code_action(&uri, range, diagnostics)
        .unwrap()
        .unwrap();

    assert_eq!(
        serde_json::to_value(actions).unwrap()[0]["title"],
        "v1",
        "the scripted server titles each action after a diagnostic it was sent"
    );
}

#[test]
fn a_closed_document_can_be_reopened_but_not_formatted() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    client.did_close(&uri).unwrap();

    let err = client.formatting(&uri).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");
    client
        .did_open(uri, "{}", lspresso::LanguageKind::JSON)
        .unwrap();
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
    let uri = lspresso::parse_uri("file:///a.json");

    let hover = client
        .request::<gen_lsp_types::HoverRequest>(gen_lsp_types::HoverParams {
            text_document_position_params: gen_lsp_types::TextDocumentPositionParams {
                text_document: gen_lsp_types::TextDocumentIdentifier { uri },
                position: lspresso::Position::new(0, 0),
            },
            work_done_progress_params: gen_lsp_types::WorkDoneProgressParams::default(),
        })
        .unwrap();
    assert_eq!(hover, None);
}

#[test]
fn generated_document_requests_require_the_document_to_be_open() {
    let mut client = spawn_scripted(3);
    let uri = lspresso::parse_uri("file:///a.json");
    let start = lspresso::Position::new(0, 0);

    let err = client.hover(&uri, start).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");

    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    assert_eq!(client.hover(&uri, start).unwrap(), None);
}

#[test]
fn the_protocol_layer_takes_raw_params_once_imported() {
    use lspresso::Protocol;

    let mut client = spawn_scripted(2);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    let hover = client
        .protocol_hover(gen_lsp_types::HoverParams {
            text_document_position_params: gen_lsp_types::TextDocumentPositionParams {
                text_document: gen_lsp_types::TextDocumentIdentifier { uri },
                position: lspresso::Position::new(0, 0),
            },
            work_done_progress_params: gen_lsp_types::WorkDoneProgressParams::default(),
        })
        .unwrap();
    assert_eq!(hover, None);
}

#[test]
fn generated_workspace_requests_need_no_open_document() {
    let mut client = spawn_scripted(2);

    let symbols = client
        .workspace_symbol(gen_lsp_types::WorkspaceSymbolParams {
            query: String::new(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(symbols, None);
}

#[test]
fn generated_document_notifications_require_the_document_to_be_open() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    let err = client.did_save(&uri).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");

    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    client.next_diagnostics(&uri, TIMEOUT).unwrap();
    client.did_save(&uri).unwrap();
    let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();
    assert_eq!(
        serde_json::to_value(&diagnostics).unwrap()[0]["message"],
        "saved",
        "the scripted server publishes \"saved\" for each save it receives"
    );
}

#[test]
fn the_escape_hatch_rejects_calls_before_initialize() {
    let mut client = scripted(&json!([]));

    let err = client
        .request::<gen_lsp_types::WorkspaceSymbolRequest>(gen_lsp_types::WorkspaceSymbolParams {
            query: String::new(),
            ..Default::default()
        })
        .unwrap_err();
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

fn first_message(diagnostics: &[lspresso::Diagnostic]) -> serde_json::Value {
    serde_json::to_value(diagnostics).unwrap()[0]["message"].clone()
}

#[test]
fn notifications_received_during_a_request_are_kept_for_later() {
    let mut client = scripted_initialized(&json!([
        { "action": "respond:0" },
        { "flood": 3, "action": "respond:0" },
    ]));
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    client.formatting(&uri).unwrap();

    let logs = client.take_notifications::<gen_lsp_types::LogMessageNotification>();
    assert_eq!(logs.len(), 3);
    assert!(
        client
            .take_notifications::<gen_lsp_types::LogMessageNotification>()
            .is_empty(),
        "taking them empties the inbox"
    );
}

#[test]
fn waiting_on_one_document_keeps_another_document_s_diagnostics() {
    let mut client = spawn_scripted(1);
    let a = lspresso::parse_uri("file:///a.json");
    let b = lspresso::parse_uri("file:///b.json");
    client
        .did_open(a.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    client
        .did_open(b.clone(), "{}", lspresso::LanguageKind::JSON)
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
fn settled_diagnostics_are_the_latest_of_several_publications() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    client.did_save(&uri).unwrap();

    let settled = client
        .settled_diagnostics(&uri, Duration::from_millis(200), TIMEOUT)
        .unwrap();
    assert_eq!(first_message(&settled), "saved");
}

#[test]
fn settling_times_out_without_any_publication_to_settle_on() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    client.next_diagnostics(&uri, TIMEOUT).unwrap();

    let err = client
        .settled_diagnostics(&uri, Duration::from_millis(50), Duration::from_millis(200))
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::DiagnosticsTimeout { .. }),
        "{err:?}"
    );
}

#[test]
fn waiting_for_cleared_diagnostics_skips_non_empty_ones() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    let err = client
        .wait_for_diagnostics_cleared(&uri, Duration::from_millis(200))
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::DiagnosticsTimeout { .. }),
        "{err:?}"
    );

    client.did_change_full(&uri, "").unwrap();
    client.wait_for_diagnostics_cleared(&uri, TIMEOUT).unwrap();
}

#[test]
fn settling_fails_when_the_deadline_cuts_the_quiet_window_short() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    let err = client
        .settled_diagnostics(&uri, Duration::from_secs(2), Duration::from_millis(200))
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::DiagnosticsTimeout { .. }),
        "{err:?}"
    );
}

#[test]
fn pulled_diagnostics_are_reused_when_the_server_reports_them_unchanged() {
    let mut client = spawn_scripted(3);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    assert_eq!(
        first_message(&client.pull_diagnostics(&uri).unwrap()),
        "pulled"
    );
    // The scripted server only says "unchanged" if sent the last result id.
    assert_eq!(
        first_message(&client.pull_diagnostics(&uri).unwrap()),
        "pulled"
    );
}

#[test]
fn waits_for_the_end_of_reported_progress() {
    let mut client = scripted_initialized(&json!([
        { "action": "respond:0" },
        { "progress": true, "action": "respond:0" },
    ]));
    client
        .workspace_symbol(gen_lsp_types::WorkspaceSymbolParams {
            query: String::new(),
            ..Default::default()
        })
        .unwrap();

    let token = client.wait_for_progress_end(TIMEOUT).unwrap();
    assert_eq!(
        token,
        gen_lsp_types::ProgressToken::String("indexing".to_owned())
    );
    let rest = client.take_notifications::<gen_lsp_types::ProgressNotification>();
    let kinds: Vec<_> = rest
        .iter()
        .map(|progress| progress.value["kind"].clone())
        .collect();
    assert_eq!(kinds, ["begin"], "only the end was taken");
}

#[test]
fn waiting_for_a_notification_reads_new_arrivals() {
    let mut client = scripted_initialized(&json!([
        { "action": "respond:0" },
        { "action": "drip" },
    ]));
    client.set_request_timeout(Duration::from_millis(50));
    client
        .workspace_symbol(gen_lsp_types::WorkspaceSymbolParams {
            query: String::new(),
            ..Default::default()
        })
        .unwrap_err();
    client.take_notifications::<gen_lsp_types::LogMessageNotification>();

    client
        .wait_for_notification::<gen_lsp_types::LogMessageNotification>(|_| true, TIMEOUT)
        .unwrap();
}

#[test]
fn taking_notifications_includes_ones_no_wait_has_read_yet() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri, "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    let start = std::time::Instant::now();
    let mut published = Vec::new();
    while published.is_empty() && start.elapsed() < TIMEOUT {
        published = client.take_notifications::<gen_lsp_types::PublishDiagnosticsNotification>();
    }
    assert!(
        !published.is_empty(),
        "the server's publications never showed up"
    );
}

#[test]
fn waiting_for_a_notification_that_never_comes_times_out() {
    let mut client = spawn_scripted(1);
    let err = client
        .wait_for_notification::<gen_lsp_types::ShowMessageNotification>(
            |_| true,
            Duration::from_millis(100),
        )
        .unwrap_err();
    assert!(
        matches!(
            err.kind(),
            ErrorKind::NotificationTimeout {
                method: "window/showMessage",
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn a_document_opens_once_until_closed() {
    let mut client = spawn_scripted(1);
    let uri = lspresso::parse_uri("file:///a.json");
    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();

    let err = client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::AlreadyOpen { .. }),
        "{err:?}"
    );

    client.did_close(&uri).unwrap();
    let err = client.did_change_full(&uri, "[]").unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotOpen { .. }), "{err:?}");
    client
        .did_open(uri, "{}", lspresso::LanguageKind::JSON)
        .unwrap();
}

#[test]
fn document_sync_before_initialize_is_rejected() {
    let mut client = scripted(&json!([]));
    let uri = lspresso::parse_uri("file:///a.json");

    let err = client
        .did_open(uri, "{}", lspresso::LanguageKind::JSON)
        .unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::NotInitialized));
}

#[test]
#[ignore = "requires a separately built target/dist/jjp or JJP_BIN"]
fn document_sync_and_diagnostics_round_trip_against_the_real_server() {
    let mut client = spawn_real_server();
    let uri = lspresso::parse_uri("file:///lspresso-doc-sync.json");

    client.initialize(lspresso::capabilities::vscode()).unwrap();

    client
        .did_open(uri.clone(), "{}", lspresso::LanguageKind::JSON)
        .unwrap();
    let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();
    assert!(
        diagnostics.is_empty(),
        "valid JSON should have no diagnostics"
    );

    client.did_change_full(&uri, "not json").unwrap();
    let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();
    assert!(
        !diagnostics.is_empty(),
        "invalid JSON should report diagnostics"
    );

    client.did_close(&uri).unwrap();

    client.shutdown().unwrap();
    let status = client.wait_for_exit(TIMEOUT).unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "requires a separately built target/dist/jjp or JJP_BIN"]
fn formatting_and_code_action_against_the_real_server() {
    let mut client = spawn_real_server();
    let uri = lspresso::parse_uri("file:///lspresso-formatting.json");

    client.initialize(lspresso::capabilities::vscode()).unwrap();

    client
        .did_open(uri.clone(), "{\"a\":1}", lspresso::LanguageKind::JSON)
        .unwrap();
    client.next_diagnostics(&uri, TIMEOUT).unwrap();

    let edits = client.formatting(&uri).unwrap();
    assert!(edits.is_some_and(|edits| !edits.is_empty()));

    client.did_change_full(&uri, "{\"a\": 1,}").unwrap();
    let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();

    let range = lspresso::Range::new(lspresso::Position::new(0, 0), lspresso::Position::new(0, 9));
    let actions = client.code_action(&uri, range, diagnostics).unwrap();
    assert!(actions.is_some_and(|actions| !actions.is_empty()));

    client.did_close(&uri).unwrap();
    client.shutdown().unwrap();
    let status = client.wait_for_exit(TIMEOUT).unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "requires a separately built target/dist/jjp or JJP_BIN"]
fn initialize_against_the_real_server() {
    let mut client = spawn_real_server();

    let result = client.initialize(lspresso::capabilities::vscode()).unwrap();
    assert_eq!(result.server_info.unwrap().name, "jjpwrgem");

    client.shutdown().unwrap();
    let status = client.wait_for_exit(TIMEOUT).unwrap();
    assert!(status.success());
}
