use std::{error::Error, process::Command, time::Duration};

use lspresso::{Client, capabilities, gen_lsp_types::*};

const TIMEOUT: Duration = Duration::from_secs(5);
type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn client(preset: fn() -> ClientCapabilities) -> TestResult<Client> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jjp"));
    command.arg("lsp");
    Ok(Client::spawn_initialized(command, preset())?.with_timeout(TIMEOUT))
}

fn open(client: &mut Client, text: &str) -> TestResult<Uri> {
    let uri: Uri = "file:///e2e.json".parse()?;
    client.did_open(uri.clone(), text, LanguageKind::JSON)?;
    Ok(uri)
}

#[test]
fn valid_json_has_no_diagnostics() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, r#"{"a": 1}"#)?;

    assert_eq!(client.diagnostics(&uri)?, []);
    Ok(())
}

#[test]
fn invalid_json_has_an_error_diagnostic() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, r#"{"a": }"#)?;

    let diagnostics = client.diagnostics(&uri)?;
    assert!(matches!(
        diagnostics.as_slice(),
        [Diagnostic {
            severity: Some(DiagnosticSeverity::Error),
            source: Some(source),
            ..
        }] if source == "jjpwrgem"
    ));
    Ok(())
}

#[test]
fn diagnostics_use_the_negotiated_encoding() -> TestResult {
    let text = "{\n  \"😀\": }";

    let mut utf16 = client(capabilities::vscode::latest)?;
    assert!(
        utf16
            .server_capabilities()
            .and_then(|capabilities| capabilities.position_encoding.as_ref())
            .is_none()
    );
    let utf16_uri = open(&mut utf16, text)?;
    let utf16 = utf16.diagnostics(&utf16_uri)?;

    let mut utf8 = client(capabilities::helix::latest)?;
    assert_eq!(
        utf8.server_capabilities()
            .and_then(|capabilities| capabilities.position_encoding.as_ref()),
        Some(&PositionEncodingKind::UTF8)
    );
    let utf8_uri = open(&mut utf8, text)?;
    let utf8 = utf8.diagnostics(&utf8_uri)?;

    let [
        Diagnostic {
            range: utf16_range, ..
        },
    ] = utf16.as_slice()
    else {
        panic!("expected one UTF-16 diagnostic: {utf16:?}");
    };
    let [
        Diagnostic {
            range: utf8_range, ..
        },
    ] = utf8.as_slice()
    else {
        panic!("expected one UTF-8 diagnostic: {utf8:?}");
    };

    assert_eq!(utf16_range.start.line, 1);
    assert_eq!(utf8_range.start.line, 1);
    assert_eq!(utf8_range.start.character, utf16_range.start.character + 2);
    assert_eq!(utf8_range.end.character, utf16_range.end.character + 2);
    Ok(())
}

#[test]
fn formatting_formats_the_document() -> TestResult {
    let input = r#"{"a":1,"b":[2,3]}"#;
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, input)?;
    assert_eq!(client.diagnostics(&uri)?, []);

    let edits = client
        .formatting(&uri, FormattingOptions::default())?
        .expect("expected formatting edits");
    let [TextEdit { new_text, .. }] = edits.as_slice() else {
        panic!("expected one formatting edit: {edits:?}");
    };
    assert_ne!(new_text, input);

    client.did_change(&uri, &edits)?;
    assert_eq!(client.document_text(&uri), Some(new_text.as_str()));
    assert_eq!(client.diagnostics(&uri)?, []);

    let again = client
        .formatting(&uri, FormattingOptions::default())?
        .expect("expected formatting edits");
    let [
        TextEdit {
            new_text: again, ..
        },
    ] = again.as_slice()
    else {
        panic!("expected one formatting edit: {again:?}");
    };
    assert_eq!(again, new_text);
    Ok(())
}

#[test]
fn formatting_is_none_for_invalid_json() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, r#"{"a": }"#)?;

    assert_eq!(client.formatting(&uri, FormattingOptions::default())?, None);
    Ok(())
}

#[test]
fn code_action_fixes_a_trailing_comma() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, r#"{"a": 1,"#)?;
    let diagnostics = client.diagnostics(&uri)?;
    let range = Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX));

    let actions = client
        .code_action(&uri, range, diagnostics)?
        .expect("expected code actions");
    let [
        CodeActionResponse::CodeAction(CodeAction {
            kind: Some(CodeActionKind::QuickFix),
            edit:
                Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..
                }),
            ..
        }),
    ] = actions.as_slice()
    else {
        panic!("expected one quickfix code action: {actions:?}");
    };

    let edits = changes.get(&uri).expect("expected edits for the document");
    client.did_change(&uri, edits)?;
    assert_eq!(client.document_text(&uri), Some(r#"{"a": 1}"#));
    assert_eq!(client.diagnostics(&uri)?, []);
    Ok(())
}
