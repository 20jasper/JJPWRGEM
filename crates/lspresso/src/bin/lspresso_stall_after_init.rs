use lspresso::server_helpers as helpers;
use serde_json::json;

fn main() {
    let (conn, _io) = lsp_server::Connection::stdio();
    helpers::handle_initialize_with(
        &conn,
        json!({"capabilities": {"textDocumentSync": {"change": 2}}}),
    );
    std::thread::sleep(std::time::Duration::MAX);
}
