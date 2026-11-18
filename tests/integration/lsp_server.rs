use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use lspresso::{ErrorKind, Protocol, capabilities, gen_lsp_types::*};

use crate::lsp_e2e::{TIMEOUT, TestResult, client};

const NOT_INITIALIZED: i64 = -32002;
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;

macro_rules! untyped_request {
    ($name:ident, $method:expr) => {
        struct $name;

        impl Request for $name {
            type Params = serde_json::Value;
            type Result = serde_json::Value;
            const METHOD: LspRequestMethod<'static> = $method;
            const MESSAGE_DIRECTION: MessageDirection = MessageDirection::ClientToServer;
        }
    };
}

untyped_request!(
    UnknownRequest,
    LspRequestMethod::Custom("workspace/somethingNew")
);
untyped_request!(
    UntypedDocumentSymbol,
    LspRequestMethod::TextDocumentDocumentSymbol
);

struct CustomNotification;

impl Notification for CustomNotification {
    type Params = serde_json::Value;
    const METHOD: LspNotificationMethod<'static> = LspNotificationMethod::Custom("$/somethingNew");
    const MESSAGE_DIRECTION: MessageDirection = MessageDirection::ClientToServer;
}

fn response_code(error: &lspresso::Error) -> Option<i32> {
    match error.kind() {
        ErrorKind::Response { code, .. } => Some(*code),
        _ => None,
    }
}

#[test]
fn exit_without_shutdown_exits_with_code_one() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    client.notify::<ExitNotification>(())?;
    assert_eq!(client.wait_for_exit()?.code(), Some(1));
    Ok(())
}

#[test]
fn shutdown_then_exit_exits_with_code_zero() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    client.shutdown()?;
    assert_eq!(client.wait_for_exit()?.code(), Some(0));
    Ok(())
}

#[test]
fn an_unknown_request_gets_method_not_found_and_an_unknown_notification_is_ignored() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    client.notify::<CustomNotification>(serde_json::json!({ "a": 1 }))?;
    let error = client
        .request::<UnknownRequest>(serde_json::Value::Null)
        .unwrap_err();
    assert_eq!(response_code(&error), Some(METHOD_NOT_FOUND));
    client.shutdown()?;
    Ok(())
}

#[test]
fn invalid_params_get_invalid_params_and_the_server_keeps_working() -> TestResult {
    let mut client = client(capabilities::vscode::latest)?;
    let error = client
        .request::<UntypedDocumentSymbol>(serde_json::json!({}))
        .unwrap_err();
    assert_eq!(response_code(&error), Some(INVALID_PARAMS));
    client.shutdown()?;
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
fn end_of_input_without_exit_exits_with_code_zero() -> TestResult {
    let mut raw = Raw::initialized()?;
    raw.close_stdin();
    assert_eq!(raw.exit_code()?, Some(0));
    Ok(())
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
