use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use gen_lsp_types::*;
use lsp_server::Message;
use lspresso::{
    Client, Protocol,
    scripted_server::{self, Config, Publications, Step},
};

macro_rules! assert_err_kind {
    ($result:expr, $($pattern:tt)+) => {{
        let err = $result.unwrap_err();
        assert!(matches!(err.kind(), $($pattern)+), "{err:?}");
    }};
}
pub(crate) use assert_err_kind;

pub mod step {
    use lsp_server::ResponseError;
    use lspresso::scripted_server::{Outcome, Step};
    use serde::Serialize;

    fn respond_with_outcome(outcome: Outcome) -> Step {
        Step::Respond {
            delay_ms: 0,
            outcome,
        }
    }

    pub fn respond() -> Step {
        respond_with_outcome(Outcome::Canned)
    }

    pub fn respond_with(result: impl Serialize) -> Step {
        respond_with_outcome(Outcome::Result(serde_json::to_value(result).unwrap()))
    }

    pub fn fail(code: impl Into<i32>, data: impl Serialize) -> Step {
        respond_with_outcome(Outcome::Error(ResponseError {
            code: code.into(),
            message: "failed".into(),
            data: Some(serde_json::to_value(data).unwrap()),
        }))
    }

    pub fn respond_after(delay_ms: u64) -> Step {
        Step::Respond {
            delay_ms,
            outcome: Outcome::Canned,
        }
    }

    pub fn send(message: impl Into<lsp_server::Message>) -> Step {
        Step::Send(message.into())
    }

    pub fn write_raw(text: &str) -> Step {
        Step::WriteRaw(text.to_owned())
    }

    pub fn exit() -> Step {
        Step::Exit
    }
}

pub struct ScriptedServer {
    config: Config,
}

impl ScriptedServer {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn steps(mut self, steps: impl IntoIterator<Item = Step>) -> Self {
        self.config.steps = steps.into_iter().collect();
        self
    }

    pub fn respond_to_all(mut self) -> Self {
        self.config.respond_to_all = true;
        self
    }

    pub fn capabilities(mut self, capabilities: ServerCapabilities) -> Self {
        self.config.capabilities = capabilities;
        self
    }

    pub fn publications(mut self, publications: Publications) -> Self {
        self.config.publications = publications;
        self
    }

    pub fn recording_to(mut self, recording: &Recording) -> Self {
        self.config.record = Some(recording.path.clone());
        self
    }

    pub fn spawn(self) -> Client {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lspresso_scripted_server"));
        command.env(
            scripted_server::ENV,
            serde_json::to_string(&self.config).unwrap(),
        );
        Client::spawn(command).expect("the scripted server should spawn")
    }

    /// Answers the initialize request itself, ahead of any `steps`.
    pub fn spawn_initialized(mut self) -> Client {
        self.config.steps.insert(0, step::respond());
        let mut client = self.spawn();
        client.initialize(ClientCapabilities::default()).unwrap();
        client
    }

    pub fn spawn_opened(self) -> (Client, Uri) {
        let mut client = self.spawn_initialized();
        let uri = a_json();
        open(&mut client, &uri);
        (client, uri)
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

    fn messages(&self) -> Vec<Message> {
        std::fs::read_to_string(&self.path)
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    pub fn requests<R: Request>(&self) -> Vec<R::Params> {
        self.messages()
            .into_iter()
            .filter_map(|message| match message {
                Message::Request(request) if request.method == R::METHOD.as_str() => {
                    Some(serde_json::from_value(request.params).unwrap())
                }
                _ => None,
            })
            .collect()
    }

    pub fn responses(&self) -> Vec<lsp_server::Response> {
        self.messages()
            .into_iter()
            .filter_map(|message| match message {
                Message::Response(response) => Some(response),
                _ => None,
            })
            .collect()
    }

    pub fn notifications<N: Notification>(&self) -> Vec<N::Params> {
        self.messages()
            .into_iter()
            .filter_map(|message| match message {
                Message::Notification(notification) => scripted_server::parse::<N>(&notification),
                _ => None,
            })
            .collect()
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        drop(std::fs::remove_file(&self.path));
    }
}

pub fn uninitialized() -> Client {
    ScriptedServer::new().spawn()
}

pub fn initialized() -> Client {
    ScriptedServer::new().respond_to_all().spawn_initialized()
}

pub fn opened() -> (Client, Uri) {
    ScriptedServer::new().respond_to_all().spawn_opened()
}

pub fn assert_not_initialized(errs: impl IntoIterator<Item = lspresso::Error>) {
    for err in errs {
        assert!(
            matches!(err.kind(), lspresso::ErrorKind::NotInitialized),
            "{err:?}"
        );
    }
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
