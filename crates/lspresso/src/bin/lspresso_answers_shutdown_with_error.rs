// Answers `shutdown` with an error response instead of success.
use lsp_server::{Connection, ErrorCode, Message, Response};
use lspresso::{
    gen_lsp_types::{Request as _, ShutdownRequest},
    test_server as helpers,
};

fn main() {
    let (conn, io) = Connection::stdio();
    helpers::handle_initialize(&conn);

    for msg in &conn.receiver {
        if let Message::Request(req) = &msg
            && req.method == ShutdownRequest::METHOD.as_str()
        {
            conn.sender
                .send(Message::Response(Response::new_err(
                    req.id.clone(),
                    ErrorCode::ServerNotInitialized as i32,
                    "server not initialized".to_owned(),
                )))
                .unwrap();
            break;
        }
    }

    // client never sends `exit` here; join to guarantee the write flushed
    drop(conn);
    drop(io.join());
}
