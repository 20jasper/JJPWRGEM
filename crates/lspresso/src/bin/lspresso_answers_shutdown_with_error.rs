// Answers `shutdown` with an error response instead of success -- used to
// test that the client surfaces a server error response as ErrorKind::Response
// with the code/message/data preserved, rather than treating any response as
// success or losing the error code.
use lsp_server::{Connection, ErrorCode, Message, Response};
use lspresso::{
    gen_lsp_types::{Request as _, ShutdownRequest},
    server_helpers as helpers,
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

    // The client treats this error response as fatal to shutdown() and
    // never sends `exit`, so there's no further message to synchronize on --
    // drop the sender to close the writer thread's channel and join it,
    // guaranteeing the response above is actually flushed before this
    // process can exit.
    drop(conn);
    drop(io.join());
}
