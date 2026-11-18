use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use insta::assert_snapshot;
use jjpwrgem_parse::{format::FORMAT_SETTINGS, settings::SettingKind};
use lspresso::{Client, ErrorKind, Initialize, capabilities, gen_lsp_types::*};
use rstest::rstest;

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

struct Raw {
    child: Child,
    stdin: Option<ChildStdin>,
    messages: Receiver<serde_json::Value>,
}

impl Raw {
    fn spawn() -> TestResult<Self> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_jjp"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take();
        let mut stdout = BufReader::new(child.stdout.take().ok_or("no stdout")?);
        let (sender, messages) = mpsc::channel();
        thread::spawn(move || {
            while let Some(message) = read_framed(&mut stdout) {
                if sender.send(message).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            messages,
        })
    }

    fn initialized() -> TestResult<Self> {
        let mut raw = Self::spawn()?;
        raw.request(1, "initialize", &serde_json::json!({ "capabilities": {} }))?;
        assert!(raw.reply()?.get("result").is_some());
        raw.notify("initialized", &serde_json::json!({}))?;
        Ok(raw)
    }

    fn send_body(&mut self, body: &str) -> TestResult {
        let stdin = self.stdin.as_mut().ok_or("stdin closed")?;
        write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len())?;
        stdin.flush()?;
        Ok(())
    }

    fn request(&mut self, id: i64, method: &str, params: &serde_json::Value) -> TestResult {
        self.send_body(
            &serde_json::json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
                .to_string(),
        )
    }

    fn notify(&mut self, method: &str, params: &serde_json::Value) -> TestResult {
        self.send_body(
            &serde_json::json!({ "jsonrpc": "2.0", "method": method, "params": params })
                .to_string(),
        )
    }

    fn reply(&self) -> TestResult<serde_json::Value> {
        loop {
            let message = self.messages.recv_timeout(TIMEOUT)?;
            if message.get("method").is_none() {
                return Ok(message);
            }
        }
    }

    fn error_code(&self) -> TestResult<(serde_json::Value, i64)> {
        let reply = self.reply()?;
        let code = reply["error"]["code"]
            .as_i64()
            .ok_or_else(|| format!("expected an error: {reply}"))?;
        Ok((reply["id"].clone(), code))
    }

    fn close_stdin(&mut self) {
        self.stdin = None;
    }

    fn exit_code(&mut self) -> TestResult<Option<i32>> {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait()? {
                return Ok(status.code());
            }
            if Instant::now() > deadline {
                self.child.kill()?;
                return Err("server did not exit".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn read_framed(input: &mut impl BufRead) -> Option<serde_json::Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            length = value.parse::<usize>().ok();
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

const NOT_INITIALIZED: i64 = -32002;
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

fn assert_still_serving(raw: &mut Raw) -> TestResult {
    raw.request(900, "shutdown", &serde_json::Value::Null)?;
    let reply = raw.reply()?;
    assert_eq!(reply["id"], 900);
    assert!(reply.get("error").is_none(), "{reply}");
    Ok(())
}

#[test]
fn a_request_before_initialize_gets_server_not_initialized() -> TestResult {
    let mut raw = Raw::spawn()?;
    raw.request(
        7,
        "textDocument/documentSymbol",
        &serde_json::json!({ "textDocument": { "uri": "file:///a.json" } }),
    )?;
    assert_eq!(raw.error_code()?, (7.into(), NOT_INITIALIZED));
    Ok(())
}

#[test]
fn a_request_after_shutdown_gets_invalid_request() -> TestResult {
    let mut raw = Raw::initialized()?;
    assert_still_serving(&mut raw)?;
    raw.request(8, "textDocument/documentSymbol", &serde_json::json!({}))?;
    assert_eq!(raw.error_code()?, (8.into(), INVALID_REQUEST));
    Ok(())
}

#[test]
fn exit_without_shutdown_exits_with_code_one() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.notify("exit", &serde_json::Value::Null)?;
    assert_eq!(raw.exit_code()?, Some(1));
    Ok(())
}

#[test]
fn shutdown_then_exit_exits_with_code_zero() -> TestResult {
    let mut raw = Raw::initialized()?;
    assert_still_serving(&mut raw)?;
    raw.notify("exit", &serde_json::Value::Null)?;
    assert_eq!(raw.exit_code()?, Some(0));
    Ok(())
}

#[test]
fn end_of_input_without_exit_exits_with_code_zero() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.close_stdin();
    assert_eq!(raw.exit_code()?, Some(0));
    Ok(())
}

#[test]
fn an_unknown_request_gets_method_not_found_and_an_unknown_notification_is_ignored() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.notify("$/somethingNew", &serde_json::json!({ "a": 1 }))?;
    raw.request(3, "workspace/somethingNew", &serde_json::Value::Null)?;
    assert_eq!(raw.error_code()?, (3.into(), METHOD_NOT_FOUND));
    assert_still_serving(&mut raw)
}

#[test]
fn invalid_params_get_invalid_params_and_the_server_keeps_working() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.request(4, "textDocument/documentSymbol", &serde_json::json!({}))?;
    assert_eq!(raw.error_code()?, (4.into(), INVALID_PARAMS));
    assert_still_serving(&mut raw)
}

#[test]
fn a_malformed_body_gets_parse_error_and_the_server_keeps_working() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.send_body(r#"{"jsonrpc": "2.0", "id": "#)?;
    assert_eq!(raw.error_code()?, (serde_json::Value::Null, PARSE_ERROR));
    raw.send_body("\u{1}\u{2}")?;
    assert_eq!(raw.error_code()?, (serde_json::Value::Null, PARSE_ERROR));
    assert_still_serving(&mut raw)
}

#[test]
fn a_body_that_is_not_a_request_gets_invalid_request_and_the_server_keeps_working() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.send_body("[1, 2]")?;
    assert_eq!(
        raw.error_code()?,
        (serde_json::Value::Null, INVALID_REQUEST)
    );
    raw.send_body(r#"{"jsonrpc": "1.0", "id": 9, "method": "shutdown"}"#)?;
    assert_eq!(raw.error_code()?, (9.into(), INVALID_REQUEST));
    assert_still_serving(&mut raw)
}

const RAW_URI: &str = "file:///raw.json";

fn open_raw() -> TestResult<Raw> {
    let mut raw = Raw::initialized()?;
    raw.notify(
        "textDocument/didOpen",
        &serde_json::json!({ "textDocument": {
            "uri": RAW_URI, "languageId": "json", "version": 1, "text": r#"{"a": [1, 2], "b": 3}"#
        } }),
    )?;
    Ok(raw)
}

fn symbol_params() -> serde_json::Value {
    serde_json::json!({ "textDocument": { "uri": RAW_URI } })
}

#[test]
fn pipelined_requests_are_answered_in_order() -> TestResult {
    let mut raw = open_raw()?;
    for id in 10..15 {
        raw.request(id, "textDocument/documentSymbol", &symbol_params())?;
    }
    for id in 10..15 {
        let reply = raw.reply()?;
        assert_eq!(reply["id"], id);
        assert_eq!(reply["result"].as_array().map(Vec::len), Some(2), "{reply}");
    }
    assert_still_serving(&mut raw)
}

#[test]
fn a_message_written_in_pieces_is_answered() -> TestResult {
    let mut raw = open_raw()?;
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 20, "method": "textDocument/documentSymbol", "params": symbol_params()
    })
    .to_string();
    let (first, second) = body.split_at(body.len() / 2);
    let stdin = raw.stdin.as_mut().ok_or("stdin closed")?;
    let pause = || thread::sleep(Duration::from_millis(50));
    write!(stdin, "Content-Length: {}\r\n\r\n", body.len())?;
    stdin.flush()?;
    pause();
    stdin.write_all(first.as_bytes())?;
    stdin.flush()?;
    pause();
    stdin.write_all(second.as_bytes())?;
    stdin.flush()?;
    let reply = raw.reply()?;
    assert_eq!(reply["id"], 20);
    assert_eq!(reply["result"].as_array().map(Vec::len), Some(2), "{reply}");
    assert_still_serving(&mut raw)
}

#[test]
fn end_of_input_with_a_request_in_flight_still_gets_the_reply() -> TestResult {
    let mut raw = open_raw()?;
    raw.request(30, "textDocument/documentSymbol", &symbol_params())?;
    raw.close_stdin();
    let reply = raw.reply()?;
    assert_eq!(reply["id"], 30);
    assert_eq!(reply["result"].as_array().map(Vec::len), Some(2), "{reply}");
    assert_eq!(raw.exit_code()?, Some(0));
    Ok(())
}
