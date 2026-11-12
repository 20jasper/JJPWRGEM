use std::{error::Error, process::Command, time::Duration};

use lspresso::{Client, ErrorKind, capabilities, gen_lsp_types::*};

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
fn initialize_reports_the_server_and_shutdown_exits_cleanly() -> TestResult {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jjp"));
    command.arg("lsp");
    let mut client = Client::spawn(command)?.with_timeout(TIMEOUT);

    let result = client.initialize(capabilities::vscode::latest())?;
    assert_eq!(
        result.server_info.map(|info| info.name).as_deref(),
        Some("jjpwrgem")
    );

    client.shutdown()?;
    assert!(client.wait_for_exit()?.success());
    Ok(())
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
fn a_pull_client_is_not_pushed_diagnostics() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    assert!(client.supports_pull_diagnostics());
    let uri = open(&mut client, r#"{"a": }"#)?;
    assert_eq!(client.diagnostics(&uri)?.len(), 1);

    let mut client = client.with_timeout(Duration::from_millis(200));
    let err = client.next_diagnostics(&uri).unwrap_err();
    assert!(
        matches!(err.kind(), ErrorKind::NotificationTimeout { .. }),
        "{err:?}"
    );
    Ok(())
}

#[test]
fn a_client_without_pull_support_is_pushed_diagnostics() -> TestResult {
    let mut client = client(ClientCapabilities::default)?;
    let uri = open(&mut client, r#"{"a": }"#)?;

    assert_eq!(client.next_diagnostics(&uri)?.len(), 1);
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

fn symbols(client: &mut Client, uri: &Uri) -> TestResult<Vec<DocumentSymbol>> {
    match client.document_symbol(uri)? {
        Some(DocumentSymbolResponse::DocumentSymbolList(symbols)) => Ok(symbols),
        other => Err(format!("expected nested document symbols: {other:?}").into()),
    }
}

fn range(start: (u32, u32), end: (u32, u32)) -> Range {
    Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1))
}

#[test]
fn document_symbols_mirror_the_document_tree() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, "{\n  \"a\": [1, \"x\"],\n  \"\": null\n}")?;

    let symbols = symbols(&mut client, &uri)?;
    let [a, empty] = symbols.as_slice() else {
        panic!("expected two top level symbols: {symbols:?}");
    };

    assert_eq!(a.name, "a");
    assert_eq!(a.kind, SymbolKind::Array);
    assert_eq!(a.detail, None);
    assert_eq!(a.selection_range, range((1, 2), (1, 5)));
    assert_eq!(a.range, range((1, 2), (1, 15)));

    let children = a.children.as_deref().expect("expected array children");
    let [number, string] = children else {
        panic!("expected two array children: {children:?}");
    };
    assert_eq!(number.name, "0");
    assert_eq!(number.kind, SymbolKind::Number);
    assert_eq!(number.detail.as_deref(), Some("1"));
    assert_eq!(number.range, range((1, 8), (1, 9)));
    assert_eq!(number.children, None);
    assert_eq!(string.name, "1");
    assert_eq!(string.kind, SymbolKind::String);
    assert_eq!(string.detail.as_deref(), Some("x"));
    assert_eq!(string.range, range((1, 11), (1, 14)));

    assert_eq!(empty.name, "\"\"");
    assert_eq!(empty.kind, SymbolKind::Null);
    assert_eq!(empty.detail.as_deref(), Some("null"));
    Ok(())
}

#[test]
fn document_symbols_use_the_negotiated_encoding() -> TestResult {
    let text = r#"{"😀": {"a": 1}}"#;

    let mut utf16 = client(capabilities::vscode::latest)?;
    let utf16_uri = open(&mut utf16, text)?;
    let utf16 = symbols(&mut utf16, &utf16_uri)?;

    let mut utf8 = client(capabilities::helix::latest)?;
    let utf8_uri = open(&mut utf8, text)?;
    let utf8 = symbols(&mut utf8, &utf8_uri)?;

    assert_eq!(utf16[0].selection_range, range((0, 1), (0, 5)));
    assert_eq!(utf8[0].selection_range, range((0, 1), (0, 7)));
    assert_eq!(utf16[0].range, range((0, 1), (0, 15)));
    assert_eq!(utf8[0].range, range((0, 1), (0, 17)));
    Ok(())
}

#[test]
fn document_symbols_past_five_thousand_are_dropped_in_preorder() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let text = format!(r#"{{"a": [{}], "b": [0]}}"#, vec!["0"; 5000].join(","));
    let uri = open(&mut client, &text)?;

    let symbols = symbols(&mut client, &uri)?;
    let [a] = symbols.as_slice() else {
        panic!("expected one top level symbol: {symbols:?}");
    };
    assert_eq!(a.children.as_ref().map(Vec::len), Some(4999));
    Ok(())
}

#[test]
fn document_symbol_details_are_capped_at_one_hundred_characters() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let long = "é".repeat(101);
    let short = "é".repeat(100);
    let text = format!(r#"{{"long": "{long}", "short": "{short}"}}"#);
    let uri = open(&mut client, &text)?;

    let symbols = symbols(&mut client, &uri)?;
    let [long_symbol, short_symbol] = symbols.as_slice() else {
        panic!("expected two top level symbols: {symbols:?}");
    };
    assert_eq!(long_symbol.detail, Some(format!("{short}…")));
    assert_eq!(short_symbol.detail, Some(short));
    Ok(())
}

#[test]
fn document_symbols_are_none_for_invalid_json() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, r#"{"a": }"#)?;

    assert_eq!(client.document_symbol(&uri)?, None);
    Ok(())
}
