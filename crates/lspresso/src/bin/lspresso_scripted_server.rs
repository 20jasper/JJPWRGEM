//! Answers each incoming request according to the next step of a script
//! passed as JSON in `LSPRESSO_SCRIPT`, for tests of the client shell.
//! Requests past the end of the script are ignored. Exits on `exit`, at
//! end of input, or when a step says to. Every document sync is answered
//! with a stale publication, then one for the version just sent; emptying
//! a document with one empty publication. `LSPRESSO_INITIALIZE_RESULT`
//! overrides the `initialize` result, `LSPRESSO_NO_DIAGNOSTICS` suppresses
//! publications, and `LSPRESSO_RECORD` names a file to append each received
//! message to, as a line of JSON.

use std::{
    io::{BufRead, BufReader, Stdout, Write, stdin, stdout},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use lsp_server::{Message, Notification, Request, Response};

struct Step {
    action: Action,
    result: Option<serde_json::Value>,
}

enum Action {
    Respond { delay_ms: u64 },
    Ignore,
    Exit,
}

/// Each step is `{"action": a}` plus an optional `"result": v` to answer
/// with instead of the default, where `a` is `"respond:<ms>"`, `"ignore"`,
/// or `"exit"`.
fn parse(step: &serde_json::Value) -> Step {
    let action = step["action"].as_str().unwrap();
    Step {
        action: match action {
            "ignore" => Action::Ignore,
            "exit" => Action::Exit,
            _ => match action.split_once(':').unwrap() {
                ("respond", ms) => Action::Respond {
                    delay_ms: ms.parse().unwrap(),
                },
                _ => panic!("unknown action {action}"),
            },
        },
        result: step.get("result").cloned(),
    }
}

fn write(out: &Mutex<Stdout>, message: &Message) {
    let mut out = out.lock().unwrap();
    message.write(&mut *out).unwrap();
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
    if std::env::var_os("LSPRESSO_NO_DIAGNOSTICS").is_some() {
        return;
    }
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

fn main() {
    let script: Vec<serde_json::Value> =
        serde_json::from_str(&std::env::var("LSPRESSO_SCRIPT").unwrap()).unwrap();
    let mut steps = script.iter().map(parse);
    let out = Arc::new(Mutex::new(stdout()));
    let mut reader = BufReader::new(stdin());

    loop {
        let request = match read(&mut reader).and_then(|message| dispatch(&out, message)) {
            Ok(Some(request)) => request,
            Ok(None) => continue,
            Err(Exit) => return,
        };
        let Some(step) = steps.next() else { continue };

        match step.action {
            Action::Respond { delay_ms } => {
                let out = Arc::clone(&out);
                let result = step.result.unwrap_or_else(|| result_for(&request));
                let response = Response::new_ok(request.id.clone(), result);
                std::thread::spawn(move || {
                    sleep(Duration::from_millis(delay_ms));
                    write(&out, &Message::Response(response));
                });
            }
            Action::Ignore => {}
            Action::Exit => std::process::exit(0),
        }
    }
}
