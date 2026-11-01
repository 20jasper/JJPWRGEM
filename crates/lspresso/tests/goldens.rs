//! What the client puts on the wire, checked against `tests/goldens/`
//! (rerun with `LSPRESSO_BLESS=1` to update them), the presets against
//! capabilities captured from real editors, and how it reads `initialize`
//! results captured from real servers, both in `tests/fixtures/`.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use gen_lsp_types::{ClientCapabilities, TextDocumentSyncKind};
use lspresso::{Client, LanguageKind, capabilities};
use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(5);

type Preset = fn() -> ClientCapabilities;

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn assert_golden(name: &str, actual: &Value) {
    let path = manifest_path(&format!("tests/goldens/{name}.json"));
    if std::env::var_os("LSPRESSO_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = serde_json::to_string_pretty(actual).unwrap() + "\n";
        std::fs::write(&path, text).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{}: {err}; run with LSPRESSO_BLESS=1", path.display()));
    let expected: Value = serde_json::from_str(&expected).unwrap();
    assert_eq!(
        *actual,
        expected,
        "{} is out of date; run with LSPRESSO_BLESS=1 if the change is intended",
        path.display()
    );
}

/// `responses` immediate answers, the first (to `initialize`) held until
/// the client answers the server's own `workspace/configuration` request.
fn script(responses: usize) -> Value {
    (0..responses)
        .map(|i| json!({ "server_request": i == 0, "action": "respond:0" }))
        .collect()
}

/// A scripted server answering `responses` requests, recording what it
/// receives to a fresh file.
fn recorded(name: &str, responses: usize, initialize_result: Option<&str>) -> (Command, PathBuf) {
    let record = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.jsonl"));
    drop(std::fs::remove_file(&record));
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command
        .env("LSPRESSO_SCRIPT", script(responses).to_string())
        .env("LSPRESSO_RECORD", &record);
    if let Some(result) = initialize_result {
        command.env("LSPRESSO_INITIALIZE_RESULT", result);
    }
    (command, record)
}

/// The recorded messages, with the one field that varies between runs
/// (the client's process id) blanked.
fn transcript(record: &Path) -> Value {
    let messages = std::fs::read_to_string(record)
        .unwrap()
        .lines()
        .map(|line| {
            let mut message: Value = serde_json::from_str(line).unwrap();
            if message["method"] == "initialize" {
                message["params"]["processId"] = Value::Null;
            }
            message
        })
        .collect();
    Value::Array(messages)
}

#[test]
fn a_session_puts_exactly_these_messages_on_the_wire() {
    let (command, record) = recorded("session", 2, None);
    let mut client = Client::spawn_initialized(command, capabilities::vscode()).unwrap();
    let uri = lspresso::parse_uri("file:///a.rs");

    client
        .did_open(uri.clone(), "fn main() {}", LanguageKind::Rust)
        .unwrap();
    client.next_diagnostics(&uri, TIMEOUT).unwrap();
    client.did_change_full(&uri, "fn main() { }").unwrap();
    client.next_diagnostics(&uri, TIMEOUT).unwrap();
    client.did_close(&uri).unwrap();
    client.shutdown().unwrap();
    client.wait_for_exit(TIMEOUT).unwrap();

    assert_golden("session", &transcript(&record));
}

/// Edits a document, then saves it, against a server declaring `sync`;
/// returns the document-sync messages it received.
fn edit_session(name: &str, sync: &Value) -> Value {
    let result =
        json!({ "capabilities": { "positionEncoding": "utf-8", "textDocumentSync": sync } });
    let (command, record) = recorded(name, 2, Some(&result.to_string()));
    let mut client = Client::spawn_initialized(command, capabilities::helix()).unwrap();
    let uri = lspresso::parse_uri("file:///a.json");
    let edit = |start: (u32, u32), end: (u32, u32), new_text: &str| lspresso::TextEdit {
        range: lspresso::Range::new(
            lspresso::Position::new(start.0, start.1),
            lspresso::Position::new(end.0, end.1),
        ),
        new_text: new_text.to_owned(),
    };

    client
        .did_open(uri.clone(), "{\"😀\": 1,\n \"b\": 2}", LanguageKind::JSON)
        .unwrap();
    client
        .did_change(
            &uri,
            &[edit((0, 9), (0, 10), "10"), edit((1, 7), (1, 7), " ")],
        )
        .unwrap();
    assert_eq!(
        client.document_text(&uri),
        Some("{\"😀\": 10,\n \"b\": 2 }"),
        "the tracked text has both edits"
    );
    client.did_save(&uri).unwrap();
    client.shutdown().unwrap();
    client.wait_for_exit(TIMEOUT).unwrap();

    transcript(&record)
        .as_array()
        .unwrap()
        .iter()
        .filter(|message| {
            message["method"]
                .as_str()
                .is_some_and(|method| method.starts_with("textDocument/"))
        })
        .cloned()
        .collect()
}

#[test]
fn edits_go_out_as_the_server_syncs_and_saves() {
    assert_golden(
        "edits",
        &json!({
            "incremental, saving text": edit_session(
                "edits-incremental",
                &json!({ "openClose": true, "change": 2, "save": { "includeText": true } }),
            ),
            "full": edit_session("edits-full", &json!(1)),
        }),
    );
}

#[test]
fn each_preset_declares_exactly_these_capabilities() {
    let presets: [(&str, Preset); 4] = [
        ("vscode", capabilities::vscode),
        ("helix", capabilities::helix),
        ("neovim", capabilities::neovim),
        ("zed", capabilities::zed),
    ];
    for (name, preset) in presets {
        let (command, record) = recorded(&format!("preset-{name}"), 1, None);
        Client::spawn_initialized(command, preset()).unwrap();

        let initialize = &transcript(&record)[0];
        assert_golden(
            &format!("capabilities/{name}"),
            &initialize["params"]["capabilities"],
        );
    }
}

/// Every value `preset` declares, `editor` declares identically -- a preset
/// may leave things out, but not claim anything its editor doesn't.
fn assert_declared_by(preset: &Value, editor: &Value, path: &str) {
    match preset {
        Value::Object(fields) => {
            for (key, value) in fields {
                assert_declared_by(value, &editor[key], &format!("{path}.{key}"));
            }
        }
        _ => assert_eq!(preset, editor, "{path} differs from the real editor's"),
    }
}

#[test]
fn presets_declare_only_what_their_real_editor_does() {
    let captures: [(&str, Preset); 1] = [("helix-25.07.1", capabilities::helix)];
    for (capture, preset) in captures {
        let path = manifest_path(&format!(
            "tests/fixtures/editor-capabilities/{capture}.json"
        ));
        let editor: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

        assert_declared_by(&serde_json::to_value(preset()).unwrap(), &editor, capture);
    }
}

fn initialized_with(fixture: &str) -> Client {
    let path = manifest_path(&format!("tests/fixtures/initialize-results/{fixture}.json"));
    let result = std::fs::read_to_string(path).unwrap();
    let (command, _) = recorded(fixture, 1, Some(&result));
    Client::spawn_initialized(command, capabilities::vscode()).unwrap()
}

#[test]
fn jjpwrgem_declares_full_sync_formatting_and_code_actions() {
    let client = initialized_with("jjpwrgem-lsp-0.1.3");

    assert_eq!(
        client.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Full)
    );
    assert!(!client.supports_incremental_sync());
    assert!(client.supports_formatting());
    assert!(client.supports_code_action());
    assert!(!client.supports_pull_diagnostics());
}

#[test]
fn vscode_json_declares_incremental_sync_and_pull_diagnostics_but_not_formatting() {
    let client = initialized_with("vscode-json-language-server-4.10.0");

    assert_eq!(
        client.text_document_sync_kind(),
        Some(TextDocumentSyncKind::Incremental)
    );
    assert!(client.supports_incremental_sync());
    // Declared as an explicit `false`, not left out.
    assert!(!client.supports_formatting());
    assert!(client.supports_code_action());
    assert!(client.supports_pull_diagnostics());
}

#[test]
fn nothing_is_supported_before_initialize() {
    let (command, _) = recorded("uninitialized", 0, None);
    let client = Client::spawn(command).unwrap();

    assert!(client.server_capabilities().is_none());
    assert!(!client.supports_formatting());
    assert_eq!(client.text_document_sync_kind(), None);
}

#[test]
fn the_capabilities_a_real_server_declared_are_kept() {
    let client = initialized_with("jjpwrgem-lsp-0.1.3");

    let path = manifest_path("tests/fixtures/initialize-results/jjpwrgem-lsp-0.1.3.json");
    let declared: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(client.server_capabilities()).unwrap(),
        declared["capabilities"],
    );
}
