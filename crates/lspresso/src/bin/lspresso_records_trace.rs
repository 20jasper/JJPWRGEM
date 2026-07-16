// Records whether $/setTrace arrived with the expected value, failing the
// shutdown response unless it did.
use lsp_server::Message;
use lspresso::test_server as helpers;

fn main() {
    let (conn, _io) = lsp_server::Connection::stdio();
    helpers::handle_initialize(&conn);

    let saw_trace = std::cell::Cell::new(false);
    helpers::run_event_loop_with_shutdown_check(
        &conn,
        |msg, _conn| {
            if let Message::Notification(n) = msg
                && n.method == "$/setTrace"
                && n.params.get("value").and_then(|v| v.as_str()) == Some("verbose")
            {
                saw_trace.set(true);
            }
        },
        || {
            if saw_trace.get() {
                Ok(())
            } else {
                Err(format!(
                    "expected setTrace(verbose); got trace={}",
                    saw_trace.get()
                ))
            }
        },
    );
}
