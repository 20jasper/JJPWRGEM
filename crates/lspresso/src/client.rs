mod transport;

use std::{
    process::{Command, ExitStatus},
    time::{Duration, Instant},
};

use crossbeam_channel::RecvTimeoutError;
use gen_lsp_types::{
    ClientCapabilities, DidChangeTextDocumentNotification, DidChangeTextDocumentParams,
    DidCloseTextDocumentNotification, DidCloseTextDocumentParams, DidOpenTextDocumentNotification,
    DidOpenTextDocumentParams, ExitNotification, InitializeRequest, InitializeResult,
    InitializedNotification, InitializedParams, LanguageKind, MessageDirection,
    Notification as LspNotification, Request as LspRequest, ServerCapabilities, ShutdownRequest,
    TextDocumentContentChangeEvent, TextDocumentContentChangeWholeDocument, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentSyncKind, Uri, VersionedTextDocumentIdentifier,
};
use lsp_server::{Message, Notification, RequestId};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

use self::transport::{Incoming, Transport};
use crate::{
    Error,
    core::{self, ClientState, DocumentTracker, Effect},
    error::{ErrorKind, elapsed_ms},
};

const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(1);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Fails to compile for a method a client must not send.
macro_rules! assert_client_to_server {
    ($item:ty) => {
        const {
            assert!(matches!(
                <$item>::MESSAGE_DIRECTION,
                MessageDirection::ClientToServer | MessageDirection::Both
            ));
        }
    };
}

#[derive(Debug)]
pub struct Client {
    transport: Transport,
    state: ClientState,
    docs: DocumentTracker,
    background_errors: Vec<Error>,
    request_timeout: Duration,
    server_capabilities: Option<ServerCapabilities>,
}

impl Client {
    pub fn spawn(command: Command) -> crate::Result<Self> {
        Ok(Self {
            transport: Transport::spawn(command)?,
            state: ClientState::default(),
            docs: DocumentTracker::default(),
            background_errors: Vec::new(),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            server_capabilities: None,
        })
    }

    pub fn spawn_initialized(
        command: Command,
        capabilities: ClientCapabilities,
    ) -> crate::Result<Self> {
        let mut client = Self::spawn(command)?;
        client.initialize(capabilities)?;
        Ok(client)
    }

    /// How long each request waits for its response. Defaults to 5s.
    pub fn set_request_timeout(&mut self, timeout: Duration) {
        self.request_timeout = timeout;
    }

    pub fn initialize(
        &mut self,
        capabilities: ClientCapabilities,
    ) -> crate::Result<InitializeResult> {
        let now = Instant::now();
        let params = core::capabilities::initialize_params(capabilities);
        let params =
            serde_json::to_value(params).expect("serializing InitializeParams as JSON cannot fail");
        let (id, message) = self
            .state
            .start_initialize(params, now, self.request_timeout)?;
        if let Err(err) = self.transport.send(&message) {
            self.state.cancel_pending_request(&id);
            return Err(err);
        }
        let result = self.wait_for_resolution(&id, InitializeRequest::METHOD.as_str())?;
        let result: InitializeResult =
            serde_json::from_value(result).map_err(|err| ErrorKind::MalformedResponse {
                method: InitializeRequest::METHOD.as_str(),
                message: err.to_string(),
            })?;

        self.server_capabilities = Some(result.capabilities.clone());
        self.send_notification::<InitializedNotification>(InitializedParams::default())?;

        Ok(result)
    }

    pub fn shutdown(&mut self) -> crate::Result<()> {
        let now = Instant::now();
        let (id, message) = self.state.start_shutdown(now, self.request_timeout)?;
        if let Err(err) = self.transport.send(&message) {
            self.state.cancel_pending_request(&id);
            return Err(err);
        }
        self.wait_for_resolution(&id, ShutdownRequest::METHOD.as_str())?;

        self.send_notification::<ExitNotification>(())?;

        Ok(())
    }

    pub fn did_open(
        &mut self,
        uri: Uri,
        text: impl Into<String>,
        language: LanguageKind,
    ) -> crate::Result<()> {
        self.state.require_initialized()?;
        self.docs.require_not_open(&uri)?;
        self.send_notification::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: language,
                version: 1,
                text: text.into(),
            },
        })?;
        self.docs.mark_open(uri);
        Ok(())
    }

    pub fn did_change_full(&mut self, uri: &Uri, text: impl Into<String>) -> crate::Result<()> {
        self.state.require_initialized()?;
        let version = self.docs.next_version(uri)?;
        self.send_notification::<DidChangeTextDocumentNotification>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                version,
                text_document_identifier: TextDocumentIdentifier { uri: uri.clone() },
            },
            content_changes: vec![TextDocumentContentChangeEvent::from(
                TextDocumentContentChangeWholeDocument { text: text.into() },
            )],
        })?;
        self.docs.commit_change(uri, version);
        Ok(())
    }

    pub fn did_close(&mut self, uri: &Uri) -> crate::Result<()> {
        self.state.require_initialized()?;
        self.docs.require_open(uri)?;
        self.send_notification::<DidCloseTextDocumentNotification>(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        })?;
        self.docs.mark_closed(uri);
        Ok(())
    }

    pub fn wait_for_exit(&mut self, timeout: Duration) -> crate::Result<ExitStatus> {
        let start = Instant::now();
        let deadline = start + timeout;
        let mut stdout_closed = false;
        loop {
            if let Some(status) = self.transport.try_wait()? {
                return Ok(status);
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(ErrorKind::ExitTimeout {
                    after_ms: elapsed_ms(start, now),
                }
                .into());
            }
            // While stdout is open, closing it is what wakes us; after that
            // there's nothing left to read, so poll the process itself.
            if stdout_closed {
                std::thread::sleep((deadline - now).min(EXIT_POLL_INTERVAL));
            } else {
                match self.pump(Some(deadline), None) {
                    Pumped::Closed(_) => stdout_closed = true,
                    Pumped::Resolved(_) | Pumped::Other | Pumped::Quiet => {}
                }
            }
        }
    }

    pub fn rss_bytes(&self) -> Option<u64> {
        let pid = Pid::from_u32(self.transport.pid());
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
        );
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        system.process(pid).map(sysinfo::Process::memory)
    }

    /// What the server declared in its `initialize` response.
    pub const fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    pub fn text_document_sync_kind(&self) -> Option<TextDocumentSyncKind> {
        self.server_capabilities()
            .and_then(core::capabilities::text_document_sync_kind)
    }

    pub fn supports_incremental_sync(&self) -> bool {
        self.text_document_sync_kind() == Some(TextDocumentSyncKind::Incremental)
    }

    pub fn background_errors(&self) -> &[Error] {
        &self.background_errors
    }

    /// Sends any request and waits for its response. Document-scoped
    /// requests aren't checked against the documents lspresso tracks.
    pub fn request<R: LspRequest>(&mut self, params: R::Params) -> crate::Result<R::Result> {
        assert_client_to_server!(R);
        let now = Instant::now();
        let timeout = self.request_timeout;
        let method = R::METHOD.as_str();
        let params =
            serde_json::to_value(params).expect("serializing request params as JSON cannot fail");
        let (id, message) = self.state.start_generic(method, params, now, timeout)?;
        if let Err(err) = self.transport.send(&message) {
            self.state.cancel_pending_request(&id);
            return Err(err);
        }
        let result = self.wait_for_resolution(&id, method)?;
        serde_json::from_value(result).map_err(|err| {
            ErrorKind::MalformedResponse {
                method,
                message: err.to_string(),
            }
            .into()
        })
    }

    /// Sends any notification. Document sync sent this way bypasses the
    /// document tracking `did_open` and friends do.
    pub fn notify<N: LspNotification>(&mut self, params: N::Params) -> crate::Result<()> {
        assert_client_to_server!(N);
        self.state.require_initialized()?;
        self.send_notification::<N>(params)
    }

    fn send_notification<N: LspNotification>(&mut self, params: N::Params) -> crate::Result<()> {
        let params = serde_json::to_value(params)
            .expect("serializing notification params as JSON cannot fail");
        self.transport
            .send(&Message::Notification(Notification::new(
                N::METHOD.as_str().to_owned(),
                params,
            )))
    }

    fn wait_for_resolution(
        &mut self,
        id: &RequestId,
        method: &'static str,
    ) -> crate::Result<serde_json::Value> {
        loop {
            match self.pump(None, Some(id)) {
                Pumped::Resolved(result) => return result,
                Pumped::Closed(closed) => {
                    self.state.cancel_pending_request(id);
                    return Err(closed.into_error(method));
                }
                Pumped::Other | Pumped::Quiet => {}
            }
        }
    }

    /// Receives until `until` or the core's next request deadline, whichever
    /// is sooner, dispatches what arrived through the core, then ticks it --
    /// after every receive, not just quiet ones, since a server streaming
    /// notifications never lets a receive time out.
    fn pump(&mut self, until: Option<Instant>, waiting_for: Option<&RequestId>) -> Pumped {
        let deadline = until.into_iter().chain(self.state.next_deadline()).min();
        let pumped = match self.transport.recv_until(deadline) {
            Ok(Incoming::Message(message)) => {
                let effects = self.state.on_message(&message);
                if let Some(result) = self.record_effects(effects, waiting_for) {
                    return Pumped::Resolved(result);
                }
                Pumped::Other
            }
            // A closed stream outranks a timeout landing on the same pass.
            Ok(Incoming::Malformed(source)) => return Pumped::Closed(Closed::Malformed(source)),
            Ok(Incoming::Disconnected) | Err(RecvTimeoutError::Disconnected) => {
                return Pumped::Closed(Closed::Disconnected);
            }
            Err(RecvTimeoutError::Timeout) => Pumped::Quiet,
        };
        let effects = self
            .state
            .process_past_deadline_requests(Instant::now());
        self.record_effects(effects, waiting_for)
            .map_or(pumped, Pumped::Resolved)
    }

    fn record_effects(
        &mut self,
        effects: Vec<Effect>,
        waiting_for: Option<&RequestId>,
    ) -> Option<crate::Result<serde_json::Value>> {
        let mut resolved = None;
        for effect in effects {
            if let Some(result) = self.record_effect(effect, waiting_for) {
                resolved = Some(result);
            }
        }
        resolved
    }

    fn record_effect(
        &mut self,
        effect: Effect,
        waiting_for: Option<&RequestId>,
    ) -> Option<crate::Result<serde_json::Value>> {
        match effect {
            Effect::Resolved(resolved_id, result) if Some(&resolved_id) == waiting_for => {
                Some(result)
            }
            Effect::Resolved(..) => None,
            Effect::Error(err) => {
                self.background_errors.push(err);
                None
            }
            Effect::Reply(message) => {
                if let Err(err) = self.transport.send(&message) {
                    self.background_errors.push(err);
                }
                None
            }
        }
    }
}

enum Pumped {
    Resolved(crate::Result<serde_json::Value>),
    Closed(Closed),
    /// A notification, a server request (already answered), or a response
    /// for no one.
    Other,
    Quiet,
}

enum Closed {
    Malformed(std::io::Error),
    Disconnected,
}

impl Closed {
    fn into_error(self, method: &'static str) -> Error {
        match self {
            Self::Malformed(source) => ErrorKind::MalformedMessage { method, source }.into(),
            Self::Disconnected => ErrorKind::Disconnected { method }.into(),
        }
    }
}
