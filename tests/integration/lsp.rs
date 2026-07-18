use lspresso::{
    Client,
    gen_lsp_types::{
        ClientCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, LanguageKind, TextDocumentContentChangeWholeDocument,
        TextDocumentIdentifier, TextDocumentItem, VersionedTextDocumentIdentifier,
    },
};

use crate::common;

#[test]
fn lsp_survives_a_document_sync_session() {
    let mut server = common::cli();
    server.arg("lsp");
    let mut client = Client::spawn(server).expect("spawn jjp lsp");
    client
        .initialize(ClientCapabilities::default())
        .expect("initialize");

    let uri = lspresso::path_to_uri(&std::env::temp_dir().join("jjpwrgem-lsp-e2e.json"));
    client
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: LanguageKind::JSON,
                version: 1,
                text: r#"{"a": 1}"#.to_owned(),
            },
        })
        .expect("did_open");
    client
        .did_change(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                version: 2,
                text_document_identifier: TextDocumentIdentifier { uri: uri.clone() },
            },
            content_changes: vec![
                TextDocumentContentChangeWholeDocument::new(r#"{"a": 2}"#.to_owned()).into(),
            ],
        })
        .expect("did_change");
    client
        .did_close(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        })
        .expect("did_close");

    client.shutdown().expect("shutdown");
    assert!(client.wait_for_exit().is_some());
}
