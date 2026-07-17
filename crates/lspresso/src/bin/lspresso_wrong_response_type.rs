// Responds to `initialize` with a value that won't deserialize into
// InitializeResult -- for testing that a malformed response surfaces as
// DeserializeError, not a panic or a silently-wrong value.
use lspresso::test_server as helpers;

fn main() {
    let (conn, _io) = lsp_server::Connection::stdio();
    helpers::handle_initialize_with(&conn, serde_json::json!(42));
    std::thread::sleep(std::time::Duration::MAX);
}
