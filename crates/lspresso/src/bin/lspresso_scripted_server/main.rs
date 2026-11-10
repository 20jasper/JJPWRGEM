use std::{
    io::{BufRead, BufReader, Stdout, Write, stdin, stdout},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use gen_lsp_types::*;
use lsp_server::{self, Message, Response};
use lspresso::scripted_server::{self, Config, Outcome, Publications, Step, notification, parse};

fn write(out: &Mutex<Stdout>, message: &Message) {
    let mut out = out.lock().unwrap();
    message.write(&mut *out).unwrap();
}

fn write_raw(out: &Mutex<Stdout>, text: &str) {
    let mut out = out.lock().unwrap();
    out.write_all(text.as_bytes()).unwrap();
    out.flush().unwrap();
}

fn canned_result(request: &lsp_server::Request, config: &Config) -> serde_json::Value {
    if request.method == InitializeRequest::METHOD.as_str() {
        serde_json::to_value(InitializeResult {
            capabilities: config.capabilities.clone(),
            server_info: None,
        })
        .unwrap()
    } else {
        serde_json::Value::Null
    }
}

fn diagnostic(message: String) -> Diagnostic {
    Diagnostic {
        message: message.into(),
        ..Default::default()
    }
}

fn publish(out: &Mutex<Stdout>, config: &Config, uri: &Uri, version: i32, text: &str) {
    let current = diagnostic(format!("v{version}"));
    let publications = match config.publications {
        Publications::Off => return,
        _ if text.is_empty() => vec![(version, vec![])],
        Publications::Unversioned => vec![(version, vec![current])],
        Publications::Versioned => vec![
            (version - 1, vec![diagnostic("stale".into())]),
            (version, vec![current]),
        ],
    };
    for (version, diagnostics) in publications {
        let params = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: (config.publications == Publications::Versioned).then_some(version),
        };
        write(out, &notification::<PublishDiagnosticsNotification>(params));
    }
}

struct Exit;
fn dispatch(
    out: &Mutex<Stdout>,
    config: &Config,
    message: Message,
) -> Result<Option<lsp_server::Request>, Exit> {
    let n = match message {
        Message::Request(request) => return Ok(Some(request)),
        Message::Notification(n) => n,
        Message::Response(_) => return Ok(None),
    };
    if n.method == ExitNotification::METHOD.as_str() {
        return Err(Exit);
    }
    if let Some(params) = parse::<DidOpenTextDocumentNotification>(&n) {
        let document = params.text_document;
        publish(out, config, &document.uri, document.version, &document.text);
    } else if let Some(params) = parse::<DidChangeTextDocumentNotification>(&n) {
        let text = match &params.content_changes[0] {
            TextDocumentContentChangeEvent::TextDocumentContentChangePartial(c) => &c.text,
            TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(c) => &c.text,
        };
        let document = params.text_document;
        publish(
            out,
            config,
            &document.text_document_identifier.uri,
            document.version,
            text,
        );
    }
    Ok(None)
}

fn read(reader: &mut impl BufRead, config: &Config) -> Result<Message, Exit> {
    let message = Message::read(reader).ok().flatten().ok_or(Exit)?;
    if let Some(path) = &config.record {
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
    let mut config: Config =
        serde_json::from_str(&std::env::var(scripted_server::ENV).unwrap()).unwrap();
    let mut steps = std::mem::take(&mut config.steps).into_iter();
    let out = Arc::new(Mutex::new(stdout()));
    let mut reader = BufReader::new(stdin());

    loop {
        let request =
            match read(&mut reader, &config).and_then(|message| dispatch(&out, &config, message)) {
                Ok(Some(request)) => request,
                Ok(None) => continue,
                Err(Exit) => return,
            };
        let step = loop {
            let step = match steps.next() {
                Some(step) => step,
                None if config.respond_to_all => Step::Respond {
                    delay_ms: 0,
                    outcome: Outcome::Canned,
                },
                None => break Step::Ignore,
            };
            match step {
                Step::Send(message) => write(&out, &message),
                Step::WriteRaw(text) => write_raw(&out, &text),
                step => break step,
            }
        };

        match step {
            Step::Respond { delay_ms, outcome } => {
                let result = match outcome {
                    Outcome::Canned => Ok(canned_result(&request, &config)),
                    Outcome::Result(result) => Ok(result),
                    Outcome::Error(error) => Err(error),
                };
                let response = Response {
                    id: request.id,
                    response_result: result,
                };
                let out = Arc::clone(&out);
                std::thread::spawn(move || {
                    sleep(Duration::from_millis(delay_ms));
                    write(&out, &Message::Response(response));
                });
            }
            Step::Send(_) | Step::WriteRaw(_) => unreachable!("handled above"),
            Step::Ignore => {}
            #[expect(clippy::infinite_loop, reason = "never reads stdin again")]
            Step::Stall => loop {
                std::thread::park();
            },
            Step::Exit => std::process::exit(0),
        }
    }
}
