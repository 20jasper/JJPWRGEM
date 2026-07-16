// Right after answering `initialize`, sends two purely-informational
// notifications ($/logTrace, telemetry/event) followed by three
// server-initiated requests (window/workDoneProgress/create,
// window/showMessageRequest, workspace/semanticTokens/refresh) -- all five
// sit queued on the client's stdin until it next blocks reading (inside
// shutdown()'s wait for its own response), exercising notification
// tolerance and server-request acknowledgment before shutdown completes.
//
// Can't reuse run_event_loop_with_shutdown_check here: that helper answers
// `shutdown` the instant it sees the request, but this fixture needs to
// defer answering until it's read the three acknowledgment responses that
// follow -- the client can't send those until it drains its own read queue,
// which only starts after it's already sent `ShutdownRequest`. Answering
// early (as the helper does) means checking the acks before they exist.
//
// The one thing worth reusing is tolerance: the client's own `initialized`
// notification arrives here before any of this, so the wait for
// ShutdownRequest below skips non-matching messages rather than assuming
// it's the first thing read.
use lsp_server::{
    Connection, ErrorCode, Message, Notification, Request, RequestId, Response, ResponseKind,
};
use lspresso::{
    gen_lsp_types::{
        MessageType, ProgressToken, Request as _, ShowMessageRequestParams, ShutdownRequest,
        WorkDoneProgressCreateParams,
    },
    server_helpers as helpers,
};
use serde_json::json;

fn main() {
    let (conn, _io) = Connection::stdio();
    helpers::handle_initialize(&conn);

    conn.sender
        .send(Message::Notification(Notification::new(
            "$/logTrace".to_owned(),
            json!({"message": "hello"}),
        )))
        .unwrap();
    conn.sender
        .send(Message::Notification(Notification::new(
            "telemetry/event".to_owned(),
            json!({"anything": true}),
        )))
        .unwrap();
    conn.sender
        .send(Message::Request(Request::new(
            RequestId::from(100),
            "window/workDoneProgress/create".to_owned(),
            serde_json::to_value(WorkDoneProgressCreateParams::new(ProgressToken::from("t")))
                .unwrap(),
        )))
        .unwrap();
    conn.sender
        .send(Message::Request(Request::new(
            RequestId::from(101),
            "window/showMessageRequest".to_owned(),
            serde_json::to_value(ShowMessageRequestParams {
                kind: MessageType::Info,
                message: "hi".to_owned(),
                actions: None,
            })
            .unwrap(),
        )))
        .unwrap();
    conn.sender
        .send(Message::Request(Request::new(
            RequestId::from(102),
            "workspace/semanticTokens/refresh".to_owned(),
            serde_json::Value::Null,
        )))
        .unwrap();

    let shutdown_id = loop {
        if let Message::Request(req) = conn.receiver.recv().unwrap()
            && req.method == ShutdownRequest::METHOD.as_str()
        {
            break req.id;
        }
    };

    let mut all_null = true;
    for expected_id in [100, 101, 102] {
        match conn.receiver.recv().unwrap() {
            Message::Response(r) if r.id == RequestId::from(expected_id) => {
                if !matches!(&r.response_kind, ResponseKind::Ok { result } if result.is_null()) {
                    all_null = false;
                }
            }
            other => panic!("expected a response for id {expected_id}, got {other:?}"),
        }
    }

    let response = if all_null {
        Response::new_ok(shutdown_id, serde_json::Value::Null)
    } else {
        Response::new_err(
            shutdown_id,
            ErrorCode::InternalError as i32,
            "one or more acknowledged requests weren't answered with null".to_owned(),
        )
    };
    conn.sender.send(Message::Response(response)).unwrap();

    // Drain the exit notification so the process ends cleanly.
    drop(conn.receiver.recv());
}
