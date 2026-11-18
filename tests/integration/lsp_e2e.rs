use std::{error::Error, process::Command, time::Duration};

use insta::assert_snapshot;
use jjpwrgem_parse::{format::FORMAT_SETTINGS, settings::SettingKind};
use lspresso::{Client, ErrorKind, Initialize, capabilities, gen_lsp_types::*};
use rstest::rstest;

const TIMEOUT: Duration = Duration::from_secs(5);
pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn client(preset: fn() -> ClientCapabilities) -> TestResult<Client> {
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

fn outline(symbols: &[DocumentSymbol]) -> String {
    fn span(r: Range) -> String {
        let (s, e) = (r.start, r.end);
        format!("{}:{}-{}:{}", s.line, s.character, e.line, e.character)
    }
    let mut out = String::new();
    for s in symbols {
        let detail = s
            .detail
            .as_ref()
            .map_or(String::new(), |d| format!(" = {d}"));
        let line = format!(
            "{} {:?} {} name {}{detail}",
            s.name,
            s.kind,
            span(s.range),
            span(s.selection_range)
        );
        out.push_str(&line);
        out.push('\n');
        for child in outline(s.children.as_deref().unwrap_or_default()).lines() {
            out += "  ";
            out += child;
            out.push('\n');
        }
    }
    out
}

#[test]
fn document_symbols_mirror_the_document_tree() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, "{\n  \"a\": [1, \"x\"],\n  \"\": null\n}")?;

    assert_snapshot!(outline(&symbols(&mut client, &uri)?), @r#"
    a Array 1:2-1:15 name 1:2-1:5
      0 Number 1:8-1:9 name 1:8-1:9 = 1
      1 String 1:11-1:14 name 1:11-1:14 = x
    "" Null 2:2-2:10 name 2:2-2:4 = null
    "#);
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

    assert_snapshot!(outline(&utf16), @"
    😀 Object 0:1-0:15 name 0:1-0:5
      a Number 0:8-0:14 name 0:8-0:11 = 1
    ");
    assert_snapshot!(outline(&utf8), @"
    😀 Object 0:1-0:17 name 0:1-0:7
      a Number 0:10-0:16 name 0:10-0:13 = 1
    ");
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

fn fold_lines(client: &mut Client, uri: &Uri) -> TestResult<String> {
    let ranges = client
        .folding_range(uri)?
        .ok_or("expected folding ranges")?;
    Ok(ranges
        .iter()
        .map(|r| format!("{}-{}", r.start_line, r.end_line))
        .collect::<Vec<_>>()
        .join(" "))
}

#[test]
fn folding_ranges_fold_multi_line_containers_once_per_start_line() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(
        &mut client,
        r#"{
  "a": {
    "b": [
      1
    ]
  },
  "c": {},
  "d": [1, 2],
  "e": {
  },
  "f": [{
    "g": 1
  }]
}"#,
    )?;

    assert_snapshot!(fold_lines(&mut client, &uri)?, @"0-12 1-4 2-3 10-11");
    Ok(())
}

#[test]
fn folding_ranges_past_five_thousand_are_dropped() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let text = format!("[{}]", vec!["[\n0\n]"; 5001].join(",\n"));
    let uri = open(&mut client, &text)?;

    let ranges = client
        .folding_range(&uri)?
        .ok_or("expected folding ranges")?;
    assert_eq!(ranges.len(), 5000);
    Ok(())
}

#[test]
fn folding_ranges_are_none_for_invalid_json() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let uri = open(&mut client, r#"{"a": }"#)?;

    assert_eq!(client.folding_range(&uri)?, None);
    Ok(())
}

const SETTINGS_INPUT: &str = r#"{"a":[1,2]}"#;

fn configured_client(options: Option<serde_json::Value>) -> TestResult<Client> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jjp"));
    command.arg("lsp");
    let mut client = Client::spawn(command)?.with_timeout(TIMEOUT);
    client.initialize(Initialize {
        capabilities: capabilities::vscode::latest(),
        options,
    })?;
    Ok(client)
}

fn formatted(client: &mut Client) -> TestResult<String> {
    let uri = open(client, SETTINGS_INPUT)?;
    let edits = client
        .formatting(&uri, FormattingOptions::default())?
        .expect("expected formatting edits");
    client.did_close(&uri)?;
    let [TextEdit { new_text, .. }] = edits.as_slice() else {
        panic!("expected one formatting edit: {edits:?}");
    };
    Ok(new_text.clone())
}

const DEFAULT_OUTPUT: &str = "{ \"a\": [1, 2] }";
const WIDTH_ZERO_LF: &str = "{\n  \"a\": [\n    1,\n    2\n  ]\n}";
const WIDTH_ZERO_CRLF: &str = "{\r\n  \"a\": [\r\n    1,\r\n    2\r\n  ]\r\n}";

fn format_options(format: serde_json::Value) -> serde_json::Value {
    serde_json::Value::Object([("format".to_owned(), format)].into_iter().collect())
}

fn format_with(format: serde_json::Value) -> TestResult<String> {
    formatted(&mut configured_client(Some(format_options(format)))?)
}

fn assert_warns_and_uses_defaults(format: serde_json::Value, fragment: &str) -> TestResult {
    let mut client = configured_client(Some(format_options(format)))?;

    let warning = client.wait_for_notification::<ShowMessageNotification>()?;
    assert_eq!(warning.kind, MessageType::Warning);
    assert!(warning.message.contains(fragment), "{}", warning.message);
    assert_eq!(formatted(&mut client)?, DEFAULT_OUTPUT);
    Ok(())
}

#[rstest]
#[case::width_and_crlf(serde_json::json!({ "preferredWidth": 0, "endOfLine": "crlf" }), WIDTH_ZERO_CRLF)]
#[case::uglify(serde_json::json!({ "mode": "uglify" }), SETTINGS_INPUT)]
#[case::uglify_ignores_width(serde_json::json!({ "mode": "uglify", "preferredWidth": 0 }), SETTINGS_INPUT)]
#[case::pretty_applies_width(serde_json::json!({ "mode": "pretty", "preferredWidth": 0 }), WIDTH_ZERO_LF)]
fn initialization_options_change_formatting(
    #[case] format: serde_json::Value,
    #[case] expected: &str,
) -> TestResult {
    assert_eq!(format_with(format)?, expected);
    Ok(())
}

#[test]
fn did_change_configuration_updates_formatting() -> TestResult {
    let mut client = configured_client(None)?;
    assert_eq!(formatted(&mut client)?, DEFAULT_OUTPUT);

    let settings = serde_json::json!({ "jjpwrgem": { "format": { "preferredWidth": 0 } } });
    client.workspace_did_change_configuration(settings)?;
    assert_eq!(formatted(&mut client)?, WIDTH_ZERO_LF);
    Ok(())
}

#[rstest]
#[case::invalid_mode(serde_json::json!({ "mode": "fancy" }), "unknown variant `fancy`")]
#[case::invalid_end_of_line(serde_json::json!({ "endOfLine": "cr" }), "unknown variant `cr`")]
fn invalid_settings_warn_and_fall_back_to_defaults(
    #[case] format: serde_json::Value,
    #[case] fragment: &str,
) -> TestResult {
    assert_warns_and_uses_defaults(format, fragment)
}

#[test]
fn table_defaults_as_client_settings_match_no_settings() -> TestResult {
    let defaults: serde_json::Map<_, _> = FORMAT_SETTINGS
        .iter()
        .map(|setting| {
            let value = match setting.kind {
                SettingKind::Integer { default } => default.into(),
                SettingKind::Choice { default, .. } => default.into(),
            };
            (setting.key(), value)
        })
        .collect();

    assert_eq!(
        format_with(defaults.into())?,
        formatted(&mut configured_client(None)?)?
    );
    Ok(())
}
