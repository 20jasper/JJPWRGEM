#![allow(
    clippy::unnecessary_self_imports,
    reason = "googletest's matches_pattern!/assert_that! expand to `use $crate::{self as googletest};` internally, not anything call sites write"
)]

use std::time::Duration;

use googletest::prelude::*;
use lspresso::{Client, ErrorKind, gen_lsp_types::*};

use crate::support::{assert_completes_within, spawn_and_init, test_uri};

#[gtest]
fn spawn_missing_binary_returns_io_error() -> Result<()> {
    let server = std::process::Command::new("lspresso-no-such-binary-xyz");
    let err = Client::spawn(server).expect_err("should fail to spawn");
    assert_that!(*err, matches_pattern!(ErrorKind::Io(_)));
    Ok(())
}

#[gtest]
fn server_crash_returns_disconnected_or_io() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_exits_immediately"));
    let mut client = Client::spawn(server)?;
    let err = client
        .initialize(ClientCapabilities::default())
        .expect_err("initialize should fail after crash");
    assert_that!(
        *err,
        any!(
            matches_pattern!(ErrorKind::ResponseDisconnected { .. }),
            matches_pattern!(ErrorKind::Io(_))
        )
    );
    Ok(())
}

#[gtest]
fn wait_for_times_out_when_server_is_silent() -> Result<()> {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_never_responds"));
    let mut client = Client::spawn(server)?.with_operation_timeout(Duration::from_millis(100));
    let err = client
        .wait_for::<PublishDiagnosticsNotification, _>(|_| true)
        .expect_err("should time out");
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::NotificationTimeout { .. })
    );
    Ok(())
}

#[test]
fn drop_does_not_deadlock_when_server_stops_reading_stdin() {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stop_reading_stdin"));
    let target_uri = test_uri();
    assert_completes_within(Duration::from_millis(3000), move || {
        let mut client = spawn_and_init(server);
        client
            .protocol_did_open_notification(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: target_uri,
                    language_id: LanguageKind::JSON,
                    version: 1,
                    text: "x".repeat(256_000),
                },
            })
            .unwrap();
        std::thread::sleep(Duration::from_millis(100));
    });
}

#[test]
fn send_does_not_block_forever_when_server_stops_reading_stdin() {
    let server = std::process::Command::new(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    let target_uri = test_uri();
    drop(assert_completes_within(
        Duration::from_millis(1000),
        move || {
            let mut client = spawn_and_init(server);
            client.protocol_did_open_notification(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: target_uri,
                    language_id: LanguageKind::JSON,
                    version: 1,
                    text: "x".repeat(100_000),
                },
            })
        },
    ));
}
