// Can't reuse run_event_loop_with_shutdown_check: it answers `shutdown` the
// instant it sees the request, but this needs to defer until it's read the
// three acks that follow (the client can't send those before it drains its
// own read queue, which only starts after it sends `ShutdownRequest`).
use lsp_server::{
    Connection, ErrorCode, Message, Notification, Request, RequestId, Response, ResponseKind,
};
use lspresso::{
    gen_lsp_types::{
        MessageType, ProgressToken, Request as _, ShowMessageRequestParams, ShutdownRequest,
        WorkDoneProgressCreateParams,
    },
    test_server as helpers,
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
