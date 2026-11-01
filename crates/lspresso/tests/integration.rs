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

    let err = workspace_symbol(&mut client).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));
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
    let err = workspace_symbol(&mut client).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));

    // Many pipe buffers' worth: a write on the caller's thread would block forever.
    client
        .notify::<gen_lsp_types::DidChangeConfigurationNotification>(
            gen_lsp_types::DidChangeConfigurationParams {
                settings: json!("x".repeat(4 * 1024 * 1024)),
            },
        )
        .unwrap();

    let err = workspace_symbol(&mut client).unwrap_err();
    assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));
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
#[ignore = "requires a separately built target/dist/jjp or JJP_BIN"]
fn initialize_against_the_real_server() {
    let mut client = spawn_real_server();

    let result = client.initialize(lspresso::capabilities::vscode()).unwrap();
    assert_eq!(result.server_info.unwrap().name, "jjpwrgem");

    client.shutdown().unwrap();
    let status = client.wait_for_exit(TIMEOUT).unwrap();
    assert!(status.success());
}
