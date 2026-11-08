use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use gen_lsp_types::*;
use lspresso::{Client, Protocol};
use serde_json::json;

pub const TIMEOUT: Duration = Duration::from_secs(5);

macro_rules! assert_kind {
    ($err:expr, $($pattern:tt)+) => {
        assert!(matches!($err.kind(), $($pattern)+), "{:?}", $err)
    };
}
pub(crate) use assert_kind;

pub struct ScriptedServer {
    command: Command,
}

impl ScriptedServer {
    pub fn new() -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
        command.env("LSPRESSO_SCRIPT", "[]");
        Self { command }
    }

    pub fn steps(mut self, steps: &serde_json::Value) -> Self {
        self.command.env("LSPRESSO_SCRIPT", steps.to_string());
        self
    }

    pub fn respond_to_all(mut self) -> Self {
        self.command.env("LSPRESSO_RESPOND_TO_ALL", "1");
        self
    }

    pub fn capabilities(mut self, capabilities: &ServerCapabilities) -> Self {
        self.command.env(
            "LSPRESSO_INITIALIZE_RESULT",
            json!({ "capabilities": capabilities }).to_string(),
        );
        self
    }

    pub fn without_diagnostics(mut self) -> Self {
        self.command.env("LSPRESSO_NO_DIAGNOSTICS", "1");
        self
    }

    pub fn unversioned(mut self) -> Self {
        self.command.env("LSPRESSO_UNVERSIONED", "1");
        self
    }

    pub fn recording_to(mut self, recording: &Recording) -> Self {
        self.command.env("LSPRESSO_RECORD", &recording.path);
        self
    }

    pub fn spawn(self) -> Client {
        Client::spawn(self.command).expect("the scripted server should spawn")
    }

    pub fn spawn_initialized(self) -> Client {
        let mut client = self.spawn();
        client.initialize(ClientCapabilities::default()).unwrap();
        client
    }
}

pub struct Recording {
    path: PathBuf,
}

impl Recording {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "lspresso-recording-{}-{}.jsonl",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        Self { path }
    }

    pub fn sent(&self, method: &str) -> Vec<serde_json::Value> {
        std::fs::read_to_string(&self.path)
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .filter(|message| message["method"] == method)
            .collect()
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        drop(std::fs::remove_file(&self.path));
    }
}

pub fn scripted(steps: &serde_json::Value) -> Client {
    ScriptedServer::new().steps(steps).spawn()
}

pub fn initialized() -> Client {
    ScriptedServer::new().respond_to_all().spawn_initialized()
}

pub fn a_json() -> Uri {
    "file:///a.json".parse().unwrap()
}

pub fn open(client: &mut Client, uri: &Uri) {
    client
        .did_open(uri.clone(), "{}", LanguageKind::JSON)
        .unwrap();
}

pub fn workspace_symbol(client: &mut Client) -> lspresso::Result<()> {
    let symbols = client.request::<WorkspaceSymbolRequest>(WorkspaceSymbolParams {
        query: String::new(),
        ..Default::default()
    })?;
    assert_eq!(symbols, None, "the scripted server answers null");
    Ok(())
}
