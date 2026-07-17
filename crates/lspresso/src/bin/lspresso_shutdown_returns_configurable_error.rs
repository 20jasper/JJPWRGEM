// Answers `shutdown` with an error whose code and message come from
// LSPRESSO_ERROR_CODE/LSPRESSO_ERROR_MESSAGE -- reusable for testing that
// any specific error code propagates through Client::shutdown faithfully.
use lsp_server::{Connection, Message, Response};
use lspresso::{
    gen_lsp_types::{Request as _, ShutdownRequest},
    test_server as helpers,
};

fn main() {
    let (conn, io) = Connection::stdio();
    helpers::handle_initialize(&conn);

    let code: i32 = std::env::var("LSPRESSO_ERROR_CODE")
        .expect("LSPRESSO_ERROR_CODE set")
        .parse()
        .expect("LSPRESSO_ERROR_CODE is an i32");
    let message = std::env::var("LSPRESSO_ERROR_MESSAGE").unwrap_or_default();

    for msg in &conn.receiver {
        if let Message::Request(req) = &msg
            && req.method == ShutdownRequest::METHOD.as_str()
        {
            conn.sender
                .send(Message::Response(Response::new_err(
                    req.id.clone(),
                    code,
                    message,
                )))
                .unwrap();
            break;
        }
    }

    // client never sends `exit` here; join to guarantee the write flushed
    drop(conn);
    drop(io.join());
}
