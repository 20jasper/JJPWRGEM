use std::{process::Command, time::Duration};

use lspresso::{
    Client, capabilities,
    gen_lsp_types::{
        ClientCapabilities, DiagnosticSeverity, FormattingOptions, LanguageKind,
        PositionEncodingKind, Uri,
    },
};

const TIMEOUT: Duration = Duration::from_secs(5);

fn client(preset: fn() -> ClientCapabilities) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jjp"));
    command.arg("lsp");
    Client::spawn_initialized(command, preset())
        .unwrap()
        .with_timeout(TIMEOUT)
}

fn open(client: &mut Client, text: &str) -> Uri {
    let uri: Uri = "file:///e2e.json".parse().unwrap();
    client
        .did_open(uri.clone(), text, LanguageKind::JSON)
        .unwrap();
    uri
}

#[test]
fn valid_json_has_no_diagnostics() {
    let mut client = client(capabilities::vscode::latest);
    let uri = open(&mut client, r#"{"a": 1}"#);

    assert_eq!(client.diagnostics(&uri).unwrap(), []);
}

#[test]
fn invalid_json_has_an_error_diagnostic() {
    let mut client = client(capabilities::vscode::latest);
    let uri = open(&mut client, r#"{"a": }"#);

    let diagnostics = client.diagnostics(&uri).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::Error));
    assert_eq!(diagnostics[0].source.as_deref(), Some("jjpwrgem"));
}

#[test]
fn diagnostics_use_the_negotiated_encoding() {
    let text = "{\n  \"😀\": }";

    let mut utf16 = client(capabilities::vscode::latest);
    assert!(
        utf16
            .server_capabilities()
            .and_then(|capabilities| capabilities.position_encoding.as_ref())
            .is_none()
    );
    let utf16_uri = open(&mut utf16, text);
    let utf16 = utf16.diagnostics(&utf16_uri).unwrap();

    let mut utf8 = client(capabilities::helix::latest);
    assert_eq!(
        utf8.server_capabilities()
            .and_then(|capabilities| capabilities.position_encoding.as_ref()),
        Some(&PositionEncodingKind::UTF8)
    );
    let utf8_uri = open(&mut utf8, text);
    let utf8 = utf8.diagnostics(&utf8_uri).unwrap();

    assert_eq!(utf16.len(), 1);
    assert_eq!(utf8.len(), 1);
    assert_eq!(utf16[0].range.start.line, 1);
    assert_eq!(utf8[0].range.start.line, 1);
    assert_eq!(
        utf8[0].range.start.character,
        utf16[0].range.start.character + 2
    );
    assert_eq!(
        utf8[0].range.end.character,
        utf16[0].range.end.character + 2
    );
}

#[test]
fn formatting_formats_the_document() {
    let input = r#"{"a":1,"b":[2,3]}"#;
    let mut client = client(capabilities::vscode::latest);
    let uri = open(&mut client, input);
    assert_eq!(client.diagnostics(&uri).unwrap(), []);

    let edits = client
        .formatting(&uri, FormattingOptions::default())
        .unwrap()
        .unwrap();
    assert_eq!(edits.len(), 1);
    assert_ne!(edits[0].new_text, input);

    client.did_change(&uri, &edits).unwrap();
    assert_eq!(client.document_text(&uri), Some(edits[0].new_text.as_str()));
    assert_eq!(client.diagnostics(&uri).unwrap(), []);

    let again = client
        .formatting(&uri, FormattingOptions::default())
        .unwrap()
        .unwrap();
    assert_eq!(again[0].new_text, edits[0].new_text);
}

#[test]
fn formatting_is_none_for_invalid_json() {
    let mut client = client(capabilities::vscode::latest);
    let uri = open(&mut client, r#"{"a": }"#);

    assert_eq!(
        client
            .formatting(&uri, FormattingOptions::default())
            .unwrap(),
        None
    );
}
