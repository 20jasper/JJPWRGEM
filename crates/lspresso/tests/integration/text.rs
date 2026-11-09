use gen_lsp_types::*;
use lspresso::{ErrorKind, Protocol};

use crate::common::*;

const OPEN_TEXT: &str = "{\"😀\": 1,\n \"b\": 2}";
const CHANGED_TEXT: &str = "{\"😀\": 10,\n \"b\": 2 }";

fn edit(start: (u32, u32), end: (u32, u32), new_text: &str) -> TextEdit {
    TextEdit {
        range: Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1)),
        new_text: new_text.to_owned(),
    }
}

fn partial(start: (u32, u32), end: (u32, u32), text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangePartial::new(
        Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1)),
        None,
        text.to_owned(),
    )
    .into()
}

fn capabilities(sync: TextDocumentSync) -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF8),
        text_document_sync: Some(sync),
        ..Default::default()
    }
}

fn incremental_capabilities() -> ServerCapabilities {
    capabilities(
        TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::Incremental),
            save: Some(Save::SaveOptions(SaveOptions {
                include_text: Some(true),
            })),
            ..Default::default()
        }
        .into(),
    )
}

type Session = (
    Vec<DidOpenTextDocumentParams>,
    Vec<DidChangeTextDocumentParams>,
    Vec<DidSaveTextDocumentParams>,
);

fn run_edit_session(capabilities: ServerCapabilities) -> Session {
    let recording = Recording::new();
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .capabilities(capabilities)
        .recording_to(&recording)
        .spawn_initialized();
    let uri = a_json();
    client
        .did_open(uri.clone(), OPEN_TEXT, LanguageKind::JSON)
        .unwrap();
    client
        .did_change(
            &uri,
            &[edit((0, 9), (0, 10), "10"), edit((1, 7), (1, 7), " ")],
        )
        .unwrap();
    assert_eq!(
        client.document_text(&uri),
        Some(CHANGED_TEXT),
        "both edits apply to the tracked text"
    );
    client.did_save(&uri).unwrap();
    client.shutdown().unwrap();
    client.wait_for_exit().unwrap();

    (
        recording.notifications::<DidOpenTextDocumentNotification>(),
        recording.notifications::<DidChangeTextDocumentNotification>(),
        recording.notifications::<DidSaveTextDocumentNotification>(),
    )
}

#[test]
fn edits_go_out_as_an_incremental_change_and_save_includes_text() {
    let (opened, changes, saves) = run_edit_session(incremental_capabilities());

    assert_eq!(opened.len(), 1);
    assert_eq!(
        opened[0].text_document,
        TextDocumentItem {
            uri: a_json(),
            language_id: LanguageKind::JSON,
            version: 1,
            text: OPEN_TEXT.into(),
        }
    );
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].text_document.version, 2);
    assert_eq!(
        changes[0].content_changes,
        vec![partial((0, 9), (1, 7), "10,\n \"b\": 2 ")]
    );
    assert_eq!(saves.len(), 1);
    assert_eq!(saves[0].text.as_deref(), Some(CHANGED_TEXT));
}

#[test]
fn edits_go_out_as_the_whole_text_when_the_server_syncs_fully() {
    let (_, changes, saves) = run_edit_session(capabilities(TextDocumentSyncKind::Full.into()));

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].text_document.version, 2);
    assert_eq!(changes[0].content_changes, vec![whole(CHANGED_TEXT)]);
    assert_eq!(saves.len(), 1);
    assert_eq!(saves[0].text.as_deref(), None);
}

#[test]
fn document_text_tracks_full_changes_and_is_forgotten_on_close() {
    let mut client = initialized_with(capabilities(TextDocumentSyncKind::Full.into()));
    let uri = a_json();
    client
        .did_open(uri.clone(), OPEN_TEXT, LanguageKind::JSON)
        .unwrap();
    client.did_change_full(&uri, CHANGED_TEXT).unwrap();
    assert_eq!(client.document_text(&uri), Some(CHANGED_TEXT));
    client.did_close(&uri).unwrap();
    assert_eq!(client.document_text(&uri), None);
}

#[test]
fn overlapping_edits_are_rejected_without_sending() {
    let mut client = initialized_with(incremental_capabilities());
    let uri = a_json();
    client
        .did_open(uri.clone(), "abc", LanguageKind::JSON)
        .unwrap();

    assert_err_kind!(
        client.did_change(&uri, &[edit((0, 0), (0, 2), ""), edit((0, 1), (0, 3), "")]),
        ErrorKind::OverlappingEdits
    );
    assert_eq!(client.document_text(&uri), Some("abc"));
}

#[test]
fn raw_incremental_changes_update_the_tracked_text() {
    let mut client = initialized_with(incremental_capabilities());
    let uri = a_json();
    client
        .did_open(uri.clone(), "abc", LanguageKind::JSON)
        .unwrap();

    client
        .protocol_did_change(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                version: 2,
                text_document_identifier: TextDocumentIdentifier { uri: uri.clone() },
            },
            content_changes: vec![partial((0, 1), (0, 2), "X")],
        })
        .unwrap();
    assert_eq!(client.document_text(&uri), Some("aXc"));
}
