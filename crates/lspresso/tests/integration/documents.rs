type TestResult<T = ()> = core::result::Result<T, Box<dyn std::error::Error>>;

use gen_lsp_types::*;
use lspresso::{Client, ErrorKind, Protocol};

use crate::common::*;

#[test]
fn a_document_opens_once_until_closed() -> TestResult {
    let (mut client, uri) = opened();
    assert_err_kind!(
        client.did_open(uri.clone(), "{}", LanguageKind::JSON),
        ErrorKind::AlreadyOpen { .. }
    );

    client.did_close(&uri)?;
    assert_err_kind!(
        client.did_change_full(&uri, "[]"),
        ErrorKind::NotOpen { .. }
    );
    open(&mut client, &uri);

    Ok(())
}

#[test]
fn raw_document_notifications_are_tracked_too() -> TestResult {
    let mut client = initialized();
    let uri = a_json();
    client.protocol_did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: LanguageKind::JSON,
            version: 7,
            text: "{}".to_owned(),
        },
    })?;
    assert_eq!(client.document_text(&uri), Some("{}"));

    assert_err_kind!(
        client.did_open(uri.clone(), "{}", LanguageKind::JSON),
        ErrorKind::AlreadyOpen { .. }
    );
    client.did_change_full(&uri, "[]")?;
    assert_eq!(client.document_text(&uri), Some("[]"));
    client.protocol_did_close(DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
    })?;
    assert_eq!(client.document_text(&uri), None);
    assert_err_kind!(
        client.did_change_full(&uri, "[]"),
        ErrorKind::NotOpen { .. }
    );

    Ok(())
}

#[test]
fn save_notifications_need_an_open_document() -> TestResult {
    let mut client = initialized();
    let uri = a_json();

    let errs = [
        client.did_save(&uri).unwrap_err(),
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

    open(&mut client, &uri);
    client.did_save(&uri)?;
    client.will_save(&uri, TextDocumentSaveReason::Manual)?;

    Ok(())
}

#[test]
fn document_sync_before_initialize_is_rejected() {
    let mut client = uninitialized();
    let uri = a_json();

    let errs = [
        client
            .did_open(uri.clone(), "{}", LanguageKind::JSON)
            .unwrap_err(),
        client.did_change_full(&uri, "[]").unwrap_err(),
        client.did_close(&uri).unwrap_err(),
        client.did_save(&uri).unwrap_err(),
        client
            .will_save(&uri, TextDocumentSaveReason::Manual)
            .unwrap_err(),
        client
            .will_save_wait_until(&uri, TextDocumentSaveReason::Manual)
            .unwrap_err(),
    ];
    assert_not_initialized(errs);
}

#[test]
fn document_sync_sends_versions_and_text() -> TestResult {
    let recording = Recording::new();
    let (mut client, uri) = ScriptedServer::new()
        .respond_to_all()
        .recording_to(&recording)
        .spawn_opened();
    client.did_change_full(&uri, "[]")?;
    client.did_change_full(&uri, "[1]")?;
    client.did_close(&uri)?;
    client.shutdown()?;
    client.wait_for_exit()?;

    let opened = recording.notifications::<DidOpenTextDocumentNotification>();
    assert_eq!(
        opened.iter().map(|p| &p.text_document).collect::<Vec<_>>(),
        [&TextDocumentItem {
            uri: uri.clone(),
            language_id: LanguageKind::JSON,
            version: 1,
            text: "{}".into(),
        }]
    );

    let changes = recording.notifications::<DidChangeTextDocumentNotification>();
    assert_eq!(
        changes
            .iter()
            .map(|p| (p.text_document.version, &p.content_changes[..]))
            .collect::<Vec<_>>(),
        [(2, &[whole("[]")][..]), (3, &[whole("[1]")][..])]
    );

    let closed = recording.notifications::<DidCloseTextDocumentNotification>();
    assert_eq!(
        closed
            .iter()
            .map(|p| &p.text_document.uri)
            .collect::<Vec<_>>(),
        [&uri]
    );

    Ok(())
}

fn save_server(include_text: bool) -> Client {
    initialized_with(ServerCapabilities {
        text_document_sync: Some(
            TextDocumentSyncOptions {
                save: Some(Save::SaveOptions(SaveOptions {
                    include_text: Some(include_text),
                })),
                ..Default::default()
            }
            .into(),
        ),
        ..Default::default()
    })
}

#[test]
fn save_text_comes_from_the_initialize_result() {
    assert!(save_server(true).supports_save_text());
    assert!(!save_server(false).supports_save_text());
}

#[test]
fn sync_kind_comes_from_the_initialize_result() {
    let incremental = initialized_with(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncKind::Incremental.into()),
        ..Default::default()
    });
    assert_eq!(
        incremental.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Incremental)
    );
    assert!(incremental.supports_incremental_sync());

    let full = initialized_with(ServerCapabilities {
        text_document_sync: Some(
            TextDocumentSyncOptions {
                change: Some(TextDocumentSyncKind::Full),
                ..Default::default()
            }
            .into(),
        ),
        ..Default::default()
    });
    assert_eq!(
        full.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Full)
    );
    assert!(!full.supports_incremental_sync());
}

#[test]
fn generic_notify_and_request_track_documents_like_the_typed_methods() -> TestResult {
    let mut client = initialized();
    let uri = a_json();
    assert_err_kind!(
        client.request::<WillSaveTextDocumentWaitUntilRequest>(WillSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            reason: TextDocumentSaveReason::Manual,
        }),
        ErrorKind::NotOpen { .. }
    );
    client.notify::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: LanguageKind::JSON,
            version: 1,
            text: "{}".to_owned(),
        },
    })?;
    assert_eq!(client.document_text(&uri), Some("{}"));
    client.did_change_full(&uri, "[]")?;
    assert_eq!(client.document_text(&uri), Some("[]"));

    Ok(())
}
