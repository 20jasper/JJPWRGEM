#![cfg(feature = "test-binaries")]
#![expect(
    clippy::unnecessary_wraps,
    reason = "tests return Result uniformly for ?, even ones that only ever build Ok(())"
)]

use std::time::Duration;

use googletest::prelude::*;
use lspresso::{Client, ErrorKind, gen_lsp_types::*};
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    drop(
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_test_writer()
            .try_init(),
    );
}

fn spawn_and_init(server: std::process::Command) -> Client {
    let mut client = Client::spawn(server).expect("spawn");
    client
        .initialize(ClientCapabilities::default())
        .expect("initialize");
    client
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
    client.notify::<ExitNotification>(&())?;
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

// lspclient[verify init-initialized-after-response]
// lspclient[verify init-capabilities-gate-features]
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

// lspclient[verify shutdown-before-exit]
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

// lspclient[verify errcode-server-not-initialized]
// lspclient[verify errcode-content-modified]
// lspclient[verify errcode-request-cancelled]
#[gtest]
fn server_error_response_surfaces_with_code_and_message() -> Result<()> {
    let server =
        std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_answers_shutdown_with_error"));
    let mut client = spawn_and_init(server);

    let err = client
        .shutdown()
        .expect_err("a server error response must surface, not be treated as success");
    let expected_message = "server not initialized".to_owned();
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::Response {
            code: eq(&-32002_i32), // ServerNotInitialized
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
// lspclient[verify refresh-client-recomputes]
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

#[gtest]
fn set_trace_arrives_as_expected() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_records_trace"));
    let mut client = spawn_and_init(server);

    client.notify::<SetTraceNotification>(&SetTraceParams {
        value: TraceValue::Verbose,
    })?;

    client.shutdown()?;
    Ok(())
}
