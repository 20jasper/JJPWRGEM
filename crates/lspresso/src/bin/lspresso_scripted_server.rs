//! Answers each incoming request according to the next step of a script
//! passed as JSON in `LSPRESSO_SCRIPT`, for tests of the client shell.
//! Requests past the end of the script are ignored. Exits on `exit`, at
//! end of input, or when a step says to. Every document sync is answered
//! with a stale publication, then one for the version just sent; emptying
//! a document with one empty publication. `LSPRESSO_INITIALIZE_RESULT`
//! overrides the `initialize` result; `LSPRESSO_RECORD` names a file to
//! append each received message to, as a line of JSON.

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Stdout, Write, stdin, stdout},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use lsp_server::{ErrorCode, Message, Notification, Request, RequestId, Response};

struct Step {
    flood: u64,
    server_request: bool,
    stray_response: bool,
    action: Action,
}

enum Action {
    Respond {
        delay_ms: u64,
    },
    /// Responds from a background thread after `delay_ms`, while the main
    /// loop keeps handling requests -- for responses that arrive late.
    Late {
        delay_ms: u64,
    },
    /// Never responds, but writes a notification every 2ms for 1.5s.
    Drip,
    Error,
    Ignore,
    /// Never reads stdin again, so the client's writes back up.
    Stall,
    Garbage,
    Exit,
}

/// Each step is `{"action": a}` plus optional `"flood": n`, `"server_request":
/// bool`, and `"stray_response": bool`, where `a` is `"respond:<ms>"`,
/// `"late:<ms>"`, `"drip"`, `"error"`, `"ignore"`, `"stall"`, `"garbage"`,
/// or `"exit"`. A server request holds the step's action until it's answered.
fn parse(step: &serde_json::Value) -> Step {
    let action = step["action"].as_str().unwrap();
    Step {
        flood: step["flood"].as_u64().unwrap_or(0),
        server_request: step["server_request"].as_bool().unwrap_or(false),
        stray_response: step["stray_response"].as_bool().unwrap_or(false),
        action: match action {
            "error" => Action::Error,
            "ignore" => Action::Ignore,
            "garbage" => Action::Garbage,
            "exit" => Action::Exit,
            "drip" => Action::Drip,
            "stall" => Action::Stall,
            _ => match action.split_once(':').unwrap() {
                ("respond", ms) => Action::Respond {
                    delay_ms: ms.parse().unwrap(),
                },
                ("late", ms) => Action::Late {
                    delay_ms: ms.parse().unwrap(),
                },
                _ => panic!("unknown action {action}"),
            },
        },
    }
}

fn write(out: &Mutex<Stdout>, message: &Message) {
    let mut out = out.lock().unwrap();
    message.write(&mut *out).unwrap();
}

fn stall() -> ! {
    loop {
        sleep(Duration::from_secs(3600));
    }
}

fn result_for(request: &Request) -> serde_json::Value {
    match request.method.as_str() {
        "initialize" => std::env::var("LSPRESSO_INITIALIZE_RESULT").map_or_else(
            |_| serde_json::json!({ "capabilities": {} }),
            |result| serde_json::from_str(&result).unwrap(),
        ),
        _ => serde_json::Value::Null,
    }
}

fn diagnostic(message: &str) -> serde_json::Value {
    serde_json::json!({
        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
        "message": message,
    })
}

fn publish(out: &Mutex<Stdout>, params: &serde_json::Value) {
    let document = &params["textDocument"];
    let text = document["text"]
        .as_str()
        .or_else(|| params["contentChanges"][0]["text"].as_str());
    let version = document["version"].as_i64().unwrap();
    let publications = if text == Some("") {
        vec![(version, vec![])]
    } else {
        vec![
            (version - 1, vec![diagnostic("stale")]),
            (version, vec![diagnostic(&format!("v{version}"))]),
        ]
    };
    for (version, diagnostics) in publications {
        write(
            out,
            &Message::Notification(Notification::new(
                "textDocument/publishDiagnostics".to_owned(),
                serde_json::json!({
                    "uri": document["uri"],
                    "version": version,
                    "diagnostics": diagnostics,
                }),
            )),
        );
    }
}

struct Exit;

/// Handles a non-request message, returning the request if it is one.
fn dispatch(out: &Mutex<Stdout>, message: Message) -> Result<Option<Request>, Exit> {
    match message {
        Message::Request(request) => return Ok(Some(request)),
        Message::Notification(n) if n.method == "exit" => return Err(Exit),
        Message::Notification(n)
            if matches!(
                n.method.as_str(),
                "textDocument/didOpen" | "textDocument/didChange"
            ) =>
        {
            publish(out, &n.params);
        }
        _ => {}
    }
    Ok(None)
}

fn read(reader: &mut impl BufRead) -> Result<Message, Exit> {
    let message = Message::read(reader).ok().flatten().ok_or(Exit)?;
    if let Ok(path) = std::env::var("LSPRESSO_RECORD") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        writeln!(file, "{}", serde_json::to_string(&message).unwrap()).unwrap();
    }
    Ok(message)
}

/// Keeps serving until the client answers `id`, queueing requests meanwhile.
fn await_reply(
    reader: &mut impl BufRead,
    out: &Mutex<Stdout>,
    id: &RequestId,
    queued: &mut VecDeque<Request>,
) -> Result<(), Exit> {
    loop {
        match read(reader)? {
            Message::Response(response) if response.id == *id => return Ok(()),
            message => queued.extend(dispatch(out, message)?),
        }
    }
}

fn main() {
    let script: Vec<serde_json::Value> =
        serde_json::from_str(&std::env::var("LSPRESSO_SCRIPT").unwrap()).unwrap();
    let mut steps = script.iter().map(parse);
    let out = Arc::new(Mutex::new(stdout()));
    let mut reader = BufReader::new(stdin());
    let mut queued = VecDeque::new();

    loop {
        let request = match queued.pop_front() {
            Some(request) => request,
            None => match read(&mut reader).and_then(|message| dispatch(&out, message)) {
                Ok(Some(request)) => request,
                Ok(None) => continue,
                Err(Exit) => return,
            },
        };
        let Some(step) = steps.next() else { continue };

        for _ in 0..step.flood {
            write(
                &out,
                &Message::Notification(Notification::new(
                    "window/logMessage".to_owned(),
                    serde_json::json!({ "type": 4, "message": "flood" }),
                )),
            );
        }
        if step.stray_response {
            write(
                &out,
                &Message::Response(Response::new_ok(
                    RequestId::from(format!("stray-{}", request.id)),
                    serde_json::Value::Null,
                )),
            );
        }

        if step.server_request {
            let id = RequestId::from(format!("server-{}", request.id));
            write(
                &out,
                &Message::Request(Request::new(
                    id.clone(),
                    "workspace/configuration".to_owned(),
                    serde_json::json!({ "items": [{ "section": "lspresso" }] }),
                )),
            );
            if let Err(Exit) = await_reply(&mut reader, &out, &id, &mut queued) {
                return;
            }
        }

        match step.action {
            Action::Respond { delay_ms } | Action::Late { delay_ms } => {
                let out = Arc::clone(&out);
                let response = Response::new_ok(request.id.clone(), result_for(&request));
                std::thread::spawn(move || {
                    sleep(Duration::from_millis(delay_ms));
                    write(&out, &Message::Response(response));
                });
            }
            Action::Error => write(
                &out,
                &Message::Response(Response::new_err(
                    request.id,
                    ErrorCode::InternalError as i32,
                    "scripted error".to_owned(),
                )),
            ),
            Action::Drip => {
                let out = Arc::clone(&out);
                std::thread::spawn(move || {
                    for _ in 0..750 {
                        write(
                            &out,
                            &Message::Notification(Notification::new(
                                "window/logMessage".to_owned(),
                                serde_json::json!({ "type": 4, "message": "drip" }),
                            )),
                        );
                        sleep(Duration::from_millis(2));
                    }
                });
            }
            Action::Ignore => {}
            Action::Stall => stall(),
            Action::Garbage => {
                let mut out = out.lock().unwrap();
                out.write_all(b"Content-Length: 8\r\n\r\nnot json").unwrap();
                out.flush().unwrap();
            }
            Action::Exit => std::process::exit(0),
        }
    }
}
