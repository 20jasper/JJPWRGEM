//! Answers each incoming request according to the next step of a script
//! passed as JSON in `LSPRESSO_SCRIPT`, for tests of the client shell.
//! Requests past the end of the script are ignored. Exits on `exit`, at
//! end of input, or when a step says to.

use std::{
    io::{BufRead, BufReader, Stdout, stdin, stdout},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use lsp_server::{Message, Request, Response};

struct Step {
    action: Action,
}

enum Action {
    Respond { delay_ms: u64 },
    Ignore,
    Exit,
}

/// Each step is `{"action": a}`, where `a` is `"respond:<ms>"`, `"ignore"`,
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
    }
}

fn write(out: &Mutex<Stdout>, message: &Message) {
    let mut out = out.lock().unwrap();
    message.write(&mut *out).unwrap();
}

fn result_for(request: &Request) -> serde_json::Value {
    match request.method.as_str() {
        "initialize" => serde_json::json!({ "capabilities": {} }),
        _ => serde_json::Value::Null,
    }
}

struct Exit;

/// Handles a non-request message, returning the request if it is one.
fn dispatch(message: Message) -> Result<Option<Request>, Exit> {
    match message {
        Message::Request(request) => return Ok(Some(request)),
        Message::Notification(n) if n.method == "exit" => return Err(Exit),
        _ => {}
    }
    Ok(None)
}

fn read(reader: &mut impl BufRead) -> Result<Message, Exit> {
    Message::read(reader).ok().flatten().ok_or(Exit)
}

fn main() {
    let script: Vec<serde_json::Value> =
        serde_json::from_str(&std::env::var("LSPRESSO_SCRIPT").unwrap()).unwrap();
    let mut steps = script.iter().map(parse);
    let out = Arc::new(Mutex::new(stdout()));
    let mut reader = BufReader::new(stdin());

    loop {
        let request = match read(&mut reader).and_then(dispatch) {
            Ok(Some(request)) => request,
            Ok(None) => continue,
            Err(Exit) => return,
        };
        let Some(step) = steps.next() else { continue };

        match step.action {
            Action::Respond { delay_ms } => {
                let out = Arc::clone(&out);
                let response = Response::new_ok(request.id.clone(), result_for(&request));
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
