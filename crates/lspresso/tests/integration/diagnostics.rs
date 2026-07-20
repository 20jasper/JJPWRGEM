#![allow(
    clippy::unnecessary_self_imports,
    reason = "googletest's matches_pattern!/assert_that! expand to `use $crate::{self as googletest};` internally, not anything call sites write"
)]

use std::time::Duration;

use googletest::prelude::*;
use lspresso::{Client, ErrorKind, gen_lsp_types::*};
use rstest::rstest;

use crate::support::{spawn_and_init, test_uri};

fn text_document_item(uri: &Uri, text: &str) -> TextDocumentItem {
    TextDocumentItem {
        uri: uri.clone(),
        language_id: LanguageKind::JSON,
        version: 1,
        text: text.to_owned(),
    }
}

fn spawn_open(bin: &'static str) -> (Client, Uri) {
    let server = std::process::Command::new(bin);
    let mut client = spawn_and_init(server).with_operation_timeout(Duration::from_millis(500));
    let uri = test_uri();
    client
        .protocol_did_open_notification(DidOpenTextDocumentParams {
            text_document: text_document_item(&uri, "{}"),
        })
        .expect("did_open");
    (client, uri)
}

// lspclient[verify cap-missing-means-absent]
#[rstest]
#[case::pull(env!("CARGO_BIN_EXE_lspresso_pulls_diagnostics"), true)]
#[case::push(env!("CARGO_BIN_EXE_lspresso_pushes_diagnostics"), false)]
#[gtest]
fn supports_pull_diagnostics_reflects_server_capability(
    #[case] bin: &'static str,
    #[case] expected: bool,
) -> Result<()> {
    let (client, _) = spawn_open(bin);
    assert_that!(client.supports_pull_diagnostics(), eq(expected));
    Ok(())
}

#[gtest]
fn wait_for_diagnostics_returns_pushed_batch() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pushes_diagnostics"));
    let diagnostics = client.wait_for_diagnostics(&uri)?;
    assert_that!(diagnostics, len(eq(1)));
    Ok(())
}

#[gtest]
fn wait_for_diagnostics_returns_pulled_batch() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pulls_diagnostics"));
    let diagnostics = client.wait_for_diagnostics(&uri)?;
    assert_that!(diagnostics, len(eq(1)));
    Ok(())
}

// lspclient[verify diag-replace-previous]
// lspclient[verify resultid-pull-diagnostics-threaded]
#[gtest]
fn next_diagnostics_reuses_cache_on_unchanged_report() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pulls_diagnostics"));
    let first = client.next_diagnostics(&uri)?;
    let second = client.next_diagnostics(&uri)?;
    assert_that!(first.clone(), len(eq(1)));
    assert_that!(second, eq(&first));
    Ok(())
}

// lspclient[verify diag-empty-clears]
#[gtest]
fn wait_for_diagnostics_cleared_completes_after_push_change() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pushes_diagnostics"));
    client.protocol_did_change_notification(DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            version: 2,
            text_document_identifier: TextDocumentIdentifier { uri: uri.clone() },
        },
        content_changes: vec![TextDocumentContentChangeWholeDocument::new("{}".to_owned()).into()],
    })?;
    client.wait_for_diagnostics_cleared(&uri)?;
    Ok(())
}

// lspclient[verify diag-empty-clears]
#[gtest]
fn wait_for_diagnostics_cleared_completes_via_pull_transition() -> Result<()> {
    let (client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pulls_then_clears_diagnostics"));
    let mut client = client.with_settle(Duration::from_millis(5));
    client.wait_for_diagnostics_cleared(&uri)?;
    Ok(())
}

#[gtest]
fn check_no_diagnostics_passes_on_silent_push_server() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_stall_after_init"));
    client.check_no_diagnostics(&uri)?;
    Ok(())
}

#[gtest]
fn check_no_diagnostics_fails_when_push_server_reports_diagnostics() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pushes_diagnostics"));
    let err = client
        .check_no_diagnostics(&uri)
        .expect_err("server pushed a diagnostic");
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::UnexpectedDiagnostics { .. })
    );
    Ok(())
}

#[gtest]
fn check_no_diagnostics_passes_on_empty_pull_server() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pulls_empty_diagnostics"));
    client.check_no_diagnostics(&uri)?;
    Ok(())
}

#[gtest]
fn check_no_diagnostics_fails_when_pull_server_reports_diagnostics() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pulls_diagnostics"));
    let err = client
        .check_no_diagnostics(&uri)
        .expect_err("server has a diagnostic on first pull");
    assert_that!(
        *err,
        matches_pattern!(ErrorKind::UnexpectedDiagnostics { .. })
    );
    Ok(())
}

// lspclient[verify sync-mandatory]
#[gtest]
fn wait_for_diagnostics_finds_notification_already_buffered_by_unrelated_wait() -> Result<()> {
    let (mut client, uri) = spawn_open(env!("CARGO_BIN_EXE_lspresso_pushes_diagnostics"));

    // Nothing pending matches SetTraceNotification, but pumping the read
    // loop while waiting for it buffers the diagnostics push the server
    // already sent instead of dropping it.
    drop(client.wait_for::<SetTraceNotification, _>(|_| true));

    let diagnostics = client.wait_for_diagnostics(&uri)?;
    assert_that!(diagnostics, len(eq(1)));
    Ok(())
}
