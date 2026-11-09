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

fn whole(text: &str) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeWholeDocument { text: text.into() }.into()
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
fn edits_go_out_as_incremental_changes_and_save_includes_text() {
    let (opened, changes, saves) = run_edit_session(incremental_capabilities());

    assert_eq!(
        opened
            .iter()
            .map(|params| &params.text_document)
            .collect::<Vec<_>>(),
        [&TextDocumentItem {
            uri: a_json(),
            language_id: LanguageKind::JSON,
            version: 1,
            text: OPEN_TEXT.into(),
        }]
    );
    assert_eq!(
        changes
            .iter()
            .map(|params| (params.text_document.version, &params.content_changes[..]))
            .collect::<Vec<_>>(),
        [(
            2,
            &[partial((1, 7), (1, 7), " "), partial((0, 9), (0, 10), "10"),][..]
        )]
    );
    assert_eq!(
        saves
            .iter()
            .map(|params| params.text.as_deref())
            .collect::<Vec<_>>(),
        [Some(CHANGED_TEXT)]
    );
}

#[test]
fn edits_go_out_as_the_whole_text_when_the_server_syncs_fully() {
    let (_, changes, saves) = run_edit_session(capabilities(TextDocumentSyncKind::Full.into()));

    assert_eq!(
        changes
            .iter()
            .map(|params| (params.text_document.version, &params.content_changes[..]))
            .collect::<Vec<_>>(),
        [(2, &[whole(CHANGED_TEXT)][..])]
    );
    assert_eq!(
        saves
            .iter()
            .map(|params| params.text.as_deref())
            .collect::<Vec<_>>(),
        [None]
    );
}

#[test]
fn document_text_tracks_full_changes_and_is_forgotten_on_close() {
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .capabilities(capabilities(TextDocumentSyncKind::Full.into()))
        .spawn_initialized();
    let uri = a_json();
    client
        .did_open(uri.clone(), OPEN_TEXT, LanguageKind::JSON)
        .unwrap();
    assert_eq!(client.document_text(&uri), Some(OPEN_TEXT));

    client.did_change_full(&uri, CHANGED_TEXT).unwrap();
    assert_eq!(client.document_text(&uri), Some(CHANGED_TEXT));

    client.did_close(&uri).unwrap();
    assert_eq!(client.document_text(&uri), None);
}

#[test]
fn overlapping_edits_are_rejected_without_sending() {
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .capabilities(incremental_capabilities())
        .spawn_initialized();
    let uri = a_json();
    client
        .did_open(uri.clone(), "abc", LanguageKind::JSON)
        .unwrap();

    let err = client
        .did_change(&uri, &[edit((0, 0), (0, 2), ""), edit((0, 1), (0, 3), "")])
        .unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::OverlappingEdits { a: 0, b: 1 }),
        "{err:?}"
    );
    assert_eq!(client.document_text(&uri), Some("abc"));

    client
        .did_change(&uri, &[edit((0, 0), (0, 1), "x")])
        .unwrap();
    assert_eq!(client.document_text(&uri), Some("xbc"));
}

#[test]
fn raw_incremental_changes_update_the_tracked_text() {
    let mut client = ScriptedServer::new()
        .respond_to_all()
        .capabilities(incremental_capabilities())
        .spawn_initialized();
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
