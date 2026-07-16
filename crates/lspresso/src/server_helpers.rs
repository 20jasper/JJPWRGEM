use gen_lsp_types::{
    ExitNotification, InitializeRequest, InitializeResult, Request as _, ShutdownRequest,
};
use lsp_server::{Connection, Message, Response};
use serde_json::Value;

trait MessageExt {
    fn as_request<R: gen_lsp_types::Request>(&self) -> Option<&lsp_server::Request>;
    fn is_notification<N: gen_lsp_types::Notification>(&self) -> bool;
}

impl MessageExt for Message {
    fn as_request<R: gen_lsp_types::Request>(&self) -> Option<&lsp_server::Request> {
        match self {
            Message::Request(req) if req.method == R::METHOD.as_str() => Some(req),
            _ => None,
        }
    }

    fn is_notification<N: gen_lsp_types::Notification>(&self) -> bool {
        matches!(self, Message::Notification(n) if n.method == N::METHOD.as_str())
    }
}

pub fn handle_initialize(conn: &Connection) {
    let result = serde_json::to_value(InitializeResult::default()).unwrap();
    handle_initialize_with(conn, result);
}

pub fn handle_initialize_with(conn: &Connection, result: Value) {
    for msg in &conn.receiver {
        let Some(req) = msg.as_request::<InitializeRequest>() else {
            continue;
        };
        conn.sender
            .send(Message::Response(Response::new_ok(req.id.clone(), result)))
            .unwrap();
        break;
    }
}

pub fn run_event_loop<F>(conn: &Connection, mut handler: F)
where
    F: FnMut(Message, &Connection),
{
    for msg in &conn.receiver {
        if msg.is_notification::<ExitNotification>() {
            return;
        }
        if let Some(req) = msg.as_request::<ShutdownRequest>() {
            conn.sender
                .send(Message::Response(Response::new_ok(
                    req.id.clone(),
                    Value::Null,
                )))
                .unwrap();
            continue;
        }
        handler(msg, conn);
    }
}

/// Like [`run_event_loop`], but `shutdown_check` decides the shutdown
/// response: `Ok(())` answers normally, `Err(message)` answers with an error
/// carrying `message`. Lets a mock server report a client-side protocol
/// violation (e.g. wrong message order or content) through the shutdown
/// response the test is already checking, instead of a side-channel file.
pub fn run_event_loop_with_shutdown_check<F, C>(
    conn: &Connection,
    mut handler: F,
    mut shutdown_check: C,
) where
    F: FnMut(Message, &Connection),
    C: FnMut() -> Result<(), String>,
{
    for msg in &conn.receiver {
        if msg.is_notification::<ExitNotification>() {
            return;
        }
        if let Some(req) = msg.as_request::<ShutdownRequest>() {
            let response = match shutdown_check() {
                Ok(()) => Response::new_ok(req.id.clone(), Value::Null),
                Err(message) => Response::new_err(
                    req.id.clone(),
                    lsp_server::ErrorCode::InternalError as i32,
                    message,
                ),
            };
            conn.sender.send(Message::Response(response)).unwrap();
            continue;
        }
        handler(msg, conn);
    }
}

pub fn handle_initialize_no_thread() {
    use std::io::{BufReader, Write};
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    loop {
        match lsp_server::Message::read(&mut reader) {
            Ok(Some(lsp_server::Message::Request(req)))
                if req.method == InitializeRequest::METHOD.as_str() =>
            {
                let result = serde_json::to_value(InitializeResult::default()).unwrap();
                let resp =
                    lsp_server::Message::Response(lsp_server::Response::new_ok(req.id, result));
                resp.write(&mut stdout).unwrap();
                stdout.flush().unwrap();
                return;
            }
            Ok(Some(_)) => {}
            _ => return,
        }
    }
}
