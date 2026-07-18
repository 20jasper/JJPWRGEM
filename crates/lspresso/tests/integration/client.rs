use std::time::Duration;

use googletest::prelude::*;
use lspresso::{Client, ErrorKind, gen_lsp_types::*};
use rstest::rstest;
use tracing_subscriber::EnvFilter;

use crate::support::{spawn_and_init, test_uri};

fn init_tracing() {
    drop(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_test_writer()
            .try_init(),
    );
}

fn spawn_initialized() -> Client {
    init_tracing();
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    spawn_and_init(server).with_operation_timeout(Duration::from_millis(100))
}

// lspclient[verify shutdown-exit-terminates]
// lspclient[verify shutdown-exit-code-reflects-shutdown]
#[gtest]
fn wait_for_exit_reflects_clean_shutdown() -> Result<()> {
    let server =
        std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_exit_code_reflects_shutdown"));
    let mut client = spawn_and_init(server);
    client.shutdown()?;
    assert_that!(client.wait_for_exit(), some(eq(0)));
    Ok(())
}

#[gtest]
fn wait_for_exit_reflects_exit_without_shutdown() -> Result<()> {
    let server =
        std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_exit_code_reflects_shutdown"));
    let mut client = spawn_and_init(server);
    client.protocol_exit_notification(())?;
    assert_that!(client.wait_for_exit(), some(eq(1)));
    Ok(())
}

#[gtest]
fn second_initialize_returns_already_initialized() -> Result<()> {
    let mut client = spawn_initialized();
    let err = client
        .initialize(ClientCapabilities::default())
        .expect_err("already initialized");
    assert_that!(*err, matches_pattern!(ErrorKind::AlreadyInitialized));
    Ok(())
}

#[gtest]
fn deser_error_on_wrong_response_type() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_wrong_response_type"));
    let mut client = Client::spawn(server)?.with_operation_timeout(Duration::from_millis(500));
    let err = client
        .request::<InitializeRequest>(InitializeParams {
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        })
        .expect_err("should fail with DeserializeError");
    assert_that!(*err, matches_pattern!(ErrorKind::DeserializeError { .. }));
    Ok(())
}

// lspclient[verify init-initialized-after-response]
#[gtest]
fn initialize_records_server_capabilities() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    let mut client = Client::spawn(server)?.with_operation_timeout(Duration::from_millis(100));
    client.initialize(ClientCapabilities::default())?;

    assert_that!(client.server_capabilities(), some(anything()));
    Ok(())
}

#[gtest]
fn rss_bytes_is_none_before_any_message_is_sampled() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    let client = Client::spawn(server)?;
    assert_that!(client.rss_bytes(), eq(None));
    Ok(())
}

#[test]
fn rss_bytes_is_some_after_a_message_is_sampled() {
    let client = spawn_initialized();
    assert_that!(client.rss_bytes(), some(anything()));
}

#[test]
fn bad_framing_surfaces_as_disconnected_not_timeout() {
    init_tracing();
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_bad_frame_after_init"));
    let mut client = spawn_and_init(server).with_operation_timeout(Duration::from_secs(10));
    let start = std::time::Instant::now();
    let err = client
        .wait_for::<PublishDiagnosticsNotification, _>(|_| true)
        .expect_err("should fail when server sends bad frame");
    assert_that!(start.elapsed(), lt(Duration::from_secs(5)));
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::NotificationDisconnected { .. })
    );
}

#[gtest]
fn wait_for_notification_times_out_on_stalled_server() -> Result<()> {
    let mut client = spawn_initialized();
    let err = client
        .wait_for::<PublishDiagnosticsNotification, _>(|_| true)
        .expect_err("should time out");
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::NotificationTimeout { .. })
    );
    Ok(())
}

// lspclient[verify reqorder-response-matches-id]
#[gtest]
fn unmatched_response_id_is_an_error() -> Result<()> {
    let server =
        std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_sends_unmatched_response_id"));
    let mut client = Client::spawn(server)?.with_operation_timeout(Duration::from_millis(500));
    client.initialize(ClientCapabilities::default())?;

    let err = client
        .shutdown()
        .expect_err("a response for an id never sent must error, not buffer forever");
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::UnmatchedResponseId { .. })
    );
    Ok(())
}

#[gtest]
fn second_shutdown_is_a_no_op() -> Result<()> {
    let server =
        std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_exit_code_reflects_shutdown"));
    let mut client = spawn_and_init(server);
    client.shutdown()?;
    // fixture only answers `shutdown` once -- a resend would hang
    client.shutdown()?;
    Ok(())
}

// lspclient[verify shutdown-before-exit]
#[gtest]
fn notify_after_shutdown_is_already_shutdown() -> Result<()> {
    let server =
        std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_exit_code_reflects_shutdown"));
    let mut client = spawn_and_init(server);
    client.shutdown()?;

    let err = client
        .protocol_set_trace_notification(SetTraceParams {
            value: TraceValue::Off,
        })
        .expect_err("no requests/notifications other than exit after shutdown");
    assert_that!(*err, matches_pattern!(ErrorKind::AlreadyShutdown));
    Ok(())
}

// lspclient[verify errcode-server-not-initialized]
// lspclient[verify errcode-content-modified]
// lspclient[verify errcode-request-cancelled]
#[rstest]
#[case::server_not_initialized(-32002, "server not initialized")]
#[case::content_modified(-32801, "content modified")]
#[case::request_cancelled(-32800, "request cancelled")]
#[gtest]
fn server_error_response_surfaces_with_code_and_message(
    #[case] code: i32,
    #[case] message: &str,
) -> Result<()> {
    let mut server = std::process::Command::new(env!(
        "CARGO_BIN_EXE_lspresso_shutdown_returns_configurable_error"
    ));
    server
        .env("LSPRESSO_ERROR_CODE", code.to_string())
        .env("LSPRESSO_ERROR_MESSAGE", message);
    let mut client = spawn_and_init(server);

    let err = client
        .shutdown()
        .expect_err("a server error response must surface, not be treated as success");
    let expected_message = message.to_owned();
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::Response {
            code: eq(&code),
            message: eq(&expected_message),
            ..
        })
    );
    Ok(())
}

// lspclient[verify trace-logtrace-tolerated]
// lspclient[verify window-telemetry-tolerated]
// lspclient[verify progress-create-acknowledged]
// lspclient[verify window-showmessagerequest-responds]
#[gtest]
fn tolerates_and_acknowledges_server_messages() -> Result<()> {
    let server = std::process::Command::new(env!(
        "CARGO_BIN_EXE_lspresso_tolerates_and_acknowledges_server_messages"
    ));
    let mut client = spawn_and_init(server);
    // fixture only answers `shutdown` if everything above was handled right
    client.shutdown()?;
    Ok(())
}

// lspclient[verify trace-client-may-set]
#[gtest]
fn set_trace_arrives_as_expected() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_records_trace"));
    let mut client = spawn_and_init(server);

    client.protocol_set_trace_notification(SetTraceParams {
        value: TraceValue::Verbose,
    })?;

    client.shutdown()?;
    Ok(())
}

fn text_document_item(uri: &Uri, text: &str) -> TextDocumentItem {
    TextDocumentItem {
        uri: uri.clone(),
        language_id: LanguageKind::JSON,
        version: 1,
        text: text.to_owned(),
    }
}

// lspclient[verify sync-mandatory]
// lspclient[verify lifecycle-didchange-only-when-open]
#[gtest]
fn doc_sync_notifications_reach_server() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_records_doc_sync"));
    let mut client = spawn_and_init(server);
    let uri = test_uri();

    client.protocol_did_open_notification(DidOpenTextDocumentParams {
        text_document: text_document_item(&uri, "v1"),
    })?;
    client.protocol_did_change_notification(DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            version: 2,
            text_document_identifier: TextDocumentIdentifier { uri: uri.clone() },
        },
        content_changes: vec![TextDocumentContentChangeWholeDocument::new("v2".to_owned()).into()],
    })?;
    client.protocol_did_close_notification(DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri },
    })?;

    client.shutdown()?;
    Ok(())
}

// lspclient[verify didopen-max-one-open]
// lspclient[verify didopen-close-open-balanced]
#[gtest]
fn did_open_twice_returns_already_open() -> Result<()> {
    let mut client = spawn_initialized();
    let uri = test_uri();
    client.protocol_did_open_notification(DidOpenTextDocumentParams {
        text_document: text_document_item(&uri, "v1"),
    })?;

    let err = client
        .protocol_did_open_notification(DidOpenTextDocumentParams {
            text_document: text_document_item(&uri, "v1"),
        })
        .expect_err("already open");
    assert_that!(*err, matches_pattern!(ErrorKind::AlreadyOpen { .. }));
    Ok(())
}

// lspclient[verify lifecycle-didchange-only-when-open]
// lspclient[verify didopen-close-open-balanced]
#[rstest]
#[case::did_change(|c: &mut Client| {
    c.protocol_did_change_notification(DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            version: 2,
            text_document_identifier: TextDocumentIdentifier { uri: test_uri() },
        },
        content_changes: vec![],
    })
})]
#[case::did_close(|c: &mut Client| {
    c.protocol_did_close_notification(DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: test_uri() },
    })
})]
#[gtest]
fn method_without_open_returns_not_open(
    #[case] call: fn(&mut Client) -> lspresso::Result<()>,
) -> Result<()> {
    let mut client = spawn_initialized();
    let err = call(&mut client).expect_err("not open");
    assert_that!(*err, matches_pattern!(ErrorKind::NotOpen { .. }));
    Ok(())
}

// lspclient[verify didopen-close-open-balanced]
#[gtest]
fn did_open_after_close_of_same_uri_succeeds() -> Result<()> {
    let mut client = spawn_initialized();
    let uri = test_uri();

    client.protocol_did_open_notification(DidOpenTextDocumentParams {
        text_document: text_document_item(&uri, "v1"),
    })?;
    client.protocol_did_close_notification(DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    })?;
    client.protocol_did_open_notification(DidOpenTextDocumentParams {
        text_document: text_document_item(&uri, "v1"),
    })?;
    Ok(())
}

// lspclient[verify uri-consistent]
#[gtest]
fn uri_matching_is_exact_not_normalized() -> Result<()> {
    let mut client = spawn_initialized();
    let uri = test_uri();
    client.protocol_did_open_notification(DidOpenTextDocumentParams {
        text_document: text_document_item(&uri, "hello"),
    })?;

    let differently_cased = lspresso::path_to_uri(&std::path::PathBuf::from(if cfg!(windows) {
        r"C:\T.json"
    } else {
        "/T.json"
    }));
    let err = client
        .protocol_did_close_notification(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: differently_cased,
            },
        })
        .expect_err("differently-cased URI must not match the opened document");
    assert_that!(*err, matches_pattern!(ErrorKind::NotOpen { .. }));
    Ok(())
}

// lspclient[verify init-first]
#[gtest]
fn did_open_can_retry_after_failing_before_initialize() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    let mut client = Client::spawn(server)?.with_operation_timeout(Duration::from_millis(100));
    let uri = test_uri();

    let err = client
        .protocol_did_open_notification(DidOpenTextDocumentParams {
            text_document: text_document_item(&uri, "v1"),
        })
        .expect_err("not initialized");
    assert_that!(*err, matches_pattern!(ErrorKind::NotInitialized));

    client.initialize(ClientCapabilities::default())?;
    client.protocol_did_open_notification(DidOpenTextDocumentParams {
        text_document: text_document_item(&uri, "v1"),
    })?;
    Ok(())
}

// lspclient[verify init-first]
#[gtest]
fn raw_request_before_initialize_bypasses_initialized_check() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    let mut client = Client::spawn(server)?.with_operation_timeout(Duration::from_millis(100));
    let err = client
        .request::<ShutdownRequest>(())
        .expect_err("should time out, not return NotInitialized");
    assert_that!(*err, matches_pattern!(ErrorKind::ResponseTimeout { .. }));
    Ok(())
}
