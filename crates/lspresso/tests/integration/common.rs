use std::{path::Path, process::Command, time::Duration};

use gen_lsp_types::*;
use lspresso::{Client, Protocol};
use serde_json::json;

pub const TIMEOUT: Duration = Duration::from_secs(5);

/// Spawns the scripted server; each step is how it answers one request.
pub fn scripted(steps: &serde_json::Value) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    Client::spawn(command).expect("the scripted server should spawn")
}

pub fn scripted_initialized(steps: &serde_json::Value) -> Client {
    let mut client = scripted(steps);
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

pub fn scripted_recording(steps: &serde_json::Value, record: &Path) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env("LSPRESSO_RECORD", record);
    Client::spawn(command).expect("the scripted server should spawn")
}

pub fn initialized_with_capabilities(capabilities: &serde_json::Value) -> Client {
    initialized_with_capabilities_and_script(capabilities, &json!([{ "action": "respond:0" }]))
}

pub fn initialized_with_capabilities_and_script(
    capabilities: &serde_json::Value,
    steps: &serde_json::Value,
) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env(
        "LSPRESSO_INITIALIZE_RESULT",
        json!({ "capabilities": capabilities }).to_string(),
    );
    let mut client = Client::spawn(command).expect("the scripted server should spawn");
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

pub fn initialized_with_capabilities_and_recording(
    capabilities: &serde_json::Value,
    steps: &serde_json::Value,
    record: &Path,
) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
    command.env("LSPRESSO_SCRIPT", steps.to_string());
    command.env(
        "LSPRESSO_INITIALIZE_RESULT",
        json!({ "capabilities": capabilities }).to_string(),
    );
    command.env("LSPRESSO_RECORD", record);
    let mut client = Client::spawn(command).expect("the scripted server should spawn");
    client.initialize(ClientCapabilities::default()).unwrap();
    client
}

pub fn a_json() -> Uri {
    "file:///a.json".parse().unwrap()
}

pub fn notification<'a>(messages: &'a [serde_json::Value], method: &str) -> &'a serde_json::Value {
    messages
        .iter()
        .find(|message| message["method"] == method)
        .unwrap_or_else(|| panic!("missing {method} notification"))
}

pub fn spawn_scripted(responses: usize) -> Client {
    scripted_initialized(&json!(vec![json!({ "action": "respond:0" }); responses]))
}

pub fn workspace_symbol(client: &mut Client) -> lspresso::Result<()> {
    let symbols = client.request::<gen_lsp_types::WorkspaceSymbolRequest>(
        gen_lsp_types::WorkspaceSymbolParams {
            query: String::new(),
            ..Default::default()
        },
    )?;
    assert_eq!(symbols, None, "the scripted server answers null");
    Ok(())
}
