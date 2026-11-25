#![cfg(target_arch = "wasm32")]

use jjpwrgem_lsp::Server;
use serde_json::{Value, json};
use wasm_bindgen_test::wasm_bindgen_test;

const URI: &str = "file:///a.json";

fn message(id: Option<u64>, method: &str, params: Value) -> String {
    let mut body = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    if let Some(id) = id {
        body["id"] = json!(id);
    }
    body.to_string()
}

fn open(text: &str) -> Server {
    let mut server = Server::new("0.0.0");
    server.handle(&message(
        Some(1),
        "initialize",
        json!({ "processId": null, "rootUri": null, "capabilities": {} }),
    ));
    server.handle(&message(None, "initialized", json!({})));
    server.handle(&message(
        None,
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": URI, "languageId": "json", "version": 1, "text": text } }),
    ));
    server
}

fn request(server: &mut Server, method: &str, params: Value) -> Value {
    let replies = server.handle(&message(Some(2), method, params));
    let reply = replies
        .iter()
        .map(|reply| serde_json::from_str::<Value>(reply).unwrap())
        .find(|reply| reply["id"] == 2)
        .unwrap();
    reply["result"].clone()
}

#[wasm_bindgen_test]
fn reports_a_syntax_error() {
    let mut server = open(r#"{"a": }"#);
    let result = request(
        &mut server,
        "textDocument/diagnostic",
        json!({ "textDocument": { "uri": URI } }),
    );
    let items = result["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0]["range"],
        json!({ "start": { "line": 0, "character": 6 }, "end": { "line": 0, "character": 7 } })
    );
    assert_eq!(items[0]["severity"], 1);
    assert_eq!(items[0]["message"], "expected json value, found `}`");
}

#[wasm_bindgen_test]
fn formats_a_document() {
    let mut server = open(r#"{"a":1}"#);
    let result = request(
        &mut server,
        "textDocument/formatting",
        json!({ "textDocument": { "uri": URI }, "options": { "tabSize": 2, "insertSpaces": true } }),
    );
    let edits = result.as_array().unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0]["newText"], "{ \"a\": 1 }");
}
