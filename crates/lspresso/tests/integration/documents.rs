use gen_lsp_types::*;
use lspresso::{Client, ErrorKind, Protocol};
use serde_json::json;

use crate::common::*;

#[test]
fn a_document_opens_once_until_closed() {
    let mut client = initialized();
    let uri = a_json();
    open(&mut client, &uri);

    let err = client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap_err();
    assert_kind!(err, ErrorKind::AlreadyOpen { .. });

    client.did_close(&uri).unwrap();
    let err = client.did_change_full(&uri, "[]").unwrap_err();
    assert_kind!(err, ErrorKind::NotOpen { .. });
    client.did_open(uri, "{}", LanguageKind::JSON).unwrap();
}

#[test]
fn raw_document_notifications_are_tracked_too() {
    let mut client = initialized();
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
    assert_kind!(err, ErrorKind::AlreadyOpen { .. });
    client.did_change_full(&uri, "[]").unwrap();
    client
        .protocol_did_close(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        })
        .unwrap();
    let err = client.did_change_full(&uri, "[]").unwrap_err();
    assert_kind!(err, ErrorKind::NotOpen { .. });
}

#[test]
fn save_notifications_need_an_open_document() {
    let mut client = initialized();
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
        assert_kind!(err, ErrorKind::NotOpen { .. });
    }

    open(&mut client, &uri);
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
        assert_kind!(err, ErrorKind::NotInitialized);
    }
}

#[test]
fn document_sync_sends_versions_and_text() {
    let recording = Recording::new();
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .recording_to(&recording)
        .spawn_initialized();

    let uri = a_json();
    open(&mut client, &uri);
    client.did_change_full(&uri, "[]").unwrap();
    client.did_change_full(&uri, "[1]").unwrap();
    client.did_close(&uri).unwrap();
    client.shutdown().unwrap();
    client.wait_for_exit(TIMEOUT).unwrap();

    let opened = &recording.sent("textDocument/didOpen")[0];
    assert_eq!(opened["params"]["textDocument"]["uri"], "file:///a.json");
    assert_eq!(opened["params"]["textDocument"]["version"], 1);
    assert_eq!(opened["params"]["textDocument"]["text"], "{}");
    assert_eq!(opened["params"]["textDocument"]["languageId"], "json");

    let changes = recording.sent("textDocument/didChange");
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0]["params"]["textDocument"]["version"], 2);
    assert_eq!(changes[0]["params"]["contentChanges"][0]["text"], "[]");
    assert_eq!(changes[1]["params"]["textDocument"]["version"], 3);
    assert_eq!(changes[1]["params"]["contentChanges"][0]["text"], "[1]");

    let closed = &recording.sent("textDocument/didClose")[0];
    assert_eq!(closed["params"]["textDocument"]["uri"], "file:///a.json");
}

fn sync_server(capabilities: &ServerCapabilities) -> Client {
    ScriptedServer::new()
        .respond_to_all()
        .capabilities(capabilities)
        .spawn_initialized()
}

#[test]
fn sync_kind_comes_from_the_initialize_result() {
    let incremental = sync_server(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncKind::Incremental.into()),
        ..Default::default()
    });
    assert_eq!(
        incremental.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Incremental)
    );
    assert!(incremental.supports_incremental_sync());

    let full = sync_server(&ServerCapabilities {
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
