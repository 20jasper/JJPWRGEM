use gen_lsp_types::*;
use lspresso::{ErrorKind, Protocol};
use serde_json::json;

use crate::common::*;

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
    let incremental = initialized_with_capabilities(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncKind::Incremental.into()),
        ..Default::default()
    });
    assert_eq!(
        incremental.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Incremental)
    );
    assert!(incremental.supports_incremental_sync());

    let full = initialized_with_capabilities(&ServerCapabilities {
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
