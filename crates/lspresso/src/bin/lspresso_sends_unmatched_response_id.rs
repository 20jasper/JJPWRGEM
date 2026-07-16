// Sends a response for an id the client never requested.
use lsp_server::{Connection, Message, Response};
use lspresso::test_server as helpers;
use serde_json::json;

fn main() {
    let (conn, _io) = Connection::stdio();
    helpers::handle_initialize(&conn);
    conn.sender
        .send(Message::Response(Response::new_ok(
            lsp_server::RequestId::from(999_999_i32),
            json!(null),
        )))
        .unwrap();
    helpers::run_event_loop(&conn, |_, _| {});
}
