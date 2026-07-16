// Exits 0 if shutdown was received before exit, 1 otherwise.
use lsp_server::Message;
use lspresso::{
    gen_lsp_types::{ExitNotification, Notification as _, Request as _, ShutdownRequest},
    test_server as helpers,
};

fn main() {
    let (conn, _io) = lsp_server::Connection::stdio();
    helpers::handle_initialize(&conn);

    let mut shutdown_received = false;
    for msg in &conn.receiver {
        match msg {
            Message::Notification(n) if n.method == ExitNotification::METHOD.as_str() => {
                std::process::exit(i32::from(!shutdown_received));
            }
            Message::Request(req) if req.method == ShutdownRequest::METHOD.as_str() => {
                shutdown_received = true;
                conn.sender
                    .send(Message::Response(lsp_server::Response::new_ok(
                        req.id,
                        serde_json::Value::Null,
                    )))
                    .unwrap();
            }
            _ => {}
        }
    }
}
