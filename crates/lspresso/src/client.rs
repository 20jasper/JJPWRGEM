mod methods;
mod transport;

use std::{
    process::{Command, ExitStatus},
    time::{Duration, Instant},
};

use crossbeam_channel::RecvTimeoutError;
use gen_lsp_types::{
    ClientCapabilities, CodeActionContext, CodeActionParams, CodeActionResponse, Diagnostic,
    DidChangeTextDocumentNotification, DidChangeTextDocumentParams,
    DidCloseTextDocumentNotification, DidCloseTextDocumentParams, DidOpenTextDocumentNotification,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentDiagnosticParams,
    DocumentFormattingParams, ExitNotification, FormattingOptions, InitializeRequest,
    InitializeResult, InitializedNotification, InitializedParams, LanguageKind, MessageDirection,
    Notification as LspNotification, ProgressNotification, ProgressToken,
    PublishDiagnosticsNotification, Range, Request as LspRequest, ServerCapabilities,
    ShutdownRequest, TextDocumentContentChangeEvent, TextDocumentContentChangeWholeDocument,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentSyncKind, TextEdit, Uri,
    VersionedTextDocumentIdentifier,
};
use lsp_server::{Message, Notification, RequestId};
use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

pub use self::methods::Protocol;
use self::transport::{Incoming, Transport};
use crate::{
    Error,
    core::{
        diagnostics::PullCache,
        inbox::{Inbox, Verdict},
        text::{self, Encoding},
        self, ClientState, DocumentTracker, Effect,
    },
    error::{ErrorKind, elapsed_ms},
};

const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(1);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const PUBLISH_DIAGNOSTICS: &str = PublishDiagnosticsNotification::METHOD.as_str();

/// Fails to compile for a method a server never sends.
macro_rules! assert_server_to_client {
    ($item:ty) => {
        const {
            assert!(matches!(
                <$item>::MESSAGE_DIRECTION,
                MessageDirection::ServerToClient | MessageDirection::Both
            ));
        }
    };
}

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
    inbox: Inbox,
    pulled: PullCache,
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
            inbox: Inbox::default(),
            pulled: PullCache::default(),
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
        let text = text.into();
        self.send_notification::<DidOpenTextDocumentNotification>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: language,
                version: 1,
                text: text.clone(),
            },
        })?;
        self.pulled.forget(&uri);
        self.docs.mark_open(uri, text);
        Ok(())
    }

    pub fn did_change_full(&mut self, uri: &Uri, text: impl Into<String>) -> crate::Result<()> {
        self.state.require_initialized()?;
        let version = self.docs.next_version(uri)?;
        let text = text.into();
        self.send_change(uri, version, vec![whole_document(text.clone())])?;
        self.docs.commit_change(uri, version, text);
        Ok(())
    }

    /// Applies `edits` to the document's text, each against it as it is,
    /// and sends the change: as those edits if the server syncs
    /// incrementally, otherwise as the whole new text.
    pub fn did_change(&mut self, uri: &Uri, edits: &[TextEdit]) -> crate::Result<()> {
        self.state.require_initialized()?;
        let version = self.docs.next_version(uri)?;
        let applied = text::apply(self.docs.text(uri)?, edits, self.position_encoding())?;
        let changes = if self.supports_incremental_sync() {
            applied.changes
        } else {
            vec![whole_document(applied.text.clone())]
        };
        self.send_change(uri, version, changes)?;
        self.docs.commit_change(uri, version, applied.text);
        Ok(())
    }

    fn send_change(
        &mut self,
        uri: &Uri,
        version: i32,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> crate::Result<()> {
        self.send_notification::<DidChangeTextDocumentNotification>(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                version,
                text_document_identifier: TextDocumentIdentifier { uri: uri.clone() },
            },
            content_changes,
        })
    }

    /// Sends a `didSave`, with the document's text if the server asked for
    /// it.
    pub fn did_save(&mut self, uri: &Uri) -> crate::Result<()> {
        self.state.require_initialized()?;
        let text = self.docs.text(uri)?;
        let text = self
            .server_capabilities()
            .is_some_and(core::capabilities::save_includes_text)
            .then(|| text.to_owned());
        self.protocol_did_save(DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            text,
        })
    }

    /// The document's text as the server was last told it.
    pub fn document_text(&self, uri: &Uri) -> Option<&str> {
        self.docs.text(uri).ok()
    }

    pub fn did_close(&mut self, uri: &Uri) -> crate::Result<()> {
        self.state.require_initialized()?;
        self.docs.require_open(uri)?;
        self.send_notification::<DidCloseTextDocumentNotification>(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        })?;
        self.pulled.forget(uri);
        self.docs.mark_closed(uri);
        Ok(())
    }

    pub fn formatting(&mut self, uri: &Uri) -> crate::Result<Option<Vec<TextEdit>>> {
        self.protocol_formatting(DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            options: FormattingOptions::default(),
            work_done_progress_params: gen_lsp_types::WorkDoneProgressParams::default(),
        })
    }

    /// `diagnostics` should be whatever the caller already has for `uri`
    /// in `range` (e.g. from `next_diagnostics`) -- real editors populate
    /// `CodeActionContext.diagnostics` the same way, and a server is free
    /// to rely on it rather than recomputing diagnostics itself.
    pub fn code_action(
        &mut self,
        uri: &Uri,
        range: Range,
        diagnostics: Vec<gen_lsp_types::Diagnostic>,
    ) -> crate::Result<Option<Vec<CodeActionResponse>>> {
        self.protocol_code_action(CodeActionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            range,
            context: CodeActionContext {
                diagnostics,
                ..Default::default()
            },
            work_done_progress_params: gen_lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: gen_lsp_types::PartialResultParams::default(),
        })
    }

    /// The next pushed diagnostics for `uri`, skipping any the server marks
    /// as for an older version than the one it was last sent. Unversioned
    /// publications always match, so after closing and reopening a URI,
    /// this may return the clear the server sent for the close.
    pub fn next_diagnostics(
        &mut self,
        uri: &Uri,
        timeout: Duration,
    ) -> crate::Result<Vec<Diagnostic>> {
        let start = Instant::now();
        let mut judge = self.diagnostics_judge(uri)?;
        self.wait_for_verdict(&mut judge, start + timeout, PUBLISH_DIAGNOSTICS)?
            .ok_or_else(|| diagnostics_timeout(uri, start))
    }

    /// The latest diagnostics for `uri` once the server has published none
    /// newer for `quiet` -- for servers that publish in stages, e.g. syntax
    /// errors first and semantic ones later.
    pub fn settled_diagnostics(
        &mut self,
        uri: &Uri,
        quiet: Duration,
        timeout: Duration,
    ) -> crate::Result<Vec<Diagnostic>> {
        let start = Instant::now();
        let deadline = start + timeout;
        let mut judge = self.diagnostics_judge(uri)?;
        let mut latest = self
            .wait_for_verdict(&mut judge, deadline, PUBLISH_DIAGNOSTICS)?
            .ok_or_else(|| diagnostics_timeout(uri, start))?;
        loop {
            let window = (Instant::now() + quiet).min(deadline);
            match self.wait_for_verdict(&mut judge, window, PUBLISH_DIAGNOSTICS)? {
                Some(newer) => latest = newer,
                None if window < deadline => return Ok(latest),
                None => return Err(diagnostics_timeout(uri, start)),
            }
        }
    }

    /// Waits for the server to publish no diagnostics for `uri`.
    pub fn wait_for_diagnostics_cleared(
        &mut self,
        uri: &Uri,
        timeout: Duration,
    ) -> crate::Result<()> {
        let start = Instant::now();
        let mut judge = self.diagnostics_judge(uri)?;
        let mut cleared = |notification: &Notification| match judge(notification) {
            Verdict::Take(diagnostics) if diagnostics.is_empty() => Verdict::Take(()),
            Verdict::Take(_) | Verdict::Discard => Verdict::Discard,
            Verdict::Keep => Verdict::Keep,
        };
        self.wait_for_verdict(&mut cleared, start + timeout, PUBLISH_DIAGNOSTICS)?
            .ok_or_else(|| diagnostics_timeout(uri, start))
    }

    /// Asks for `uri`'s diagnostics with `textDocument/diagnostic`, letting
    /// the server answer "unchanged" since the last pull.
    pub fn pull_diagnostics(&mut self, uri: &Uri) -> crate::Result<Vec<Diagnostic>> {
        let report = self.protocol_diagnostic(DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            identifier: None,
            previous_result_id: self.pulled.previous_result_id(uri),
            work_done_progress_params: gen_lsp_types::WorkDoneProgressParams::default(),
            partial_result_params: gen_lsp_types::PartialResultParams::default(),
        })?;
        self.pulled.on_report(uri, report)
    }

    /// The next `N` from the server whose params `matches` accepts, taken
    /// from those already received if one is waiting.
    pub fn wait_for_notification<N: LspNotification>(
        &mut self,
        mut matches: impl FnMut(&N::Params) -> bool,
        timeout: Duration,
    ) -> crate::Result<N::Params> {
        assert_server_to_client!(N);
        let start = Instant::now();
        let method = N::METHOD.as_str();
        let mut judge = |notification: &Notification| match parse::<N>(notification) {
            Some(params) if matches(&params) => Verdict::Take(params),
            _ => Verdict::Keep,
        };
        self.wait_for_verdict(&mut judge, start + timeout, method)?
            .ok_or_else(|| {
                ErrorKind::NotificationTimeout {
                    method,
                    after_ms: elapsed_ms(start, Instant::now()),
                }
                .into()
            })
    }

    /// Every `N` received so far that no wait has taken, oldest first.
    pub fn take_notifications<N: LspNotification>(&mut self) -> Vec<N::Params> {
        assert_server_to_client!(N);
        self.stash_ready();
        self.inbox.drain(parse::<N>)
    }

    /// Waits for the next `$/progress` that ends a piece of work, e.g. a
    /// server's initial indexing, and returns its token.
    pub fn wait_for_progress_end(&mut self, timeout: Duration) -> crate::Result<ProgressToken> {
        self.wait_for_notification::<ProgressNotification>(
            |progress| progress.value["kind"] == "end",
            timeout,
        )
        .map(|progress| progress.token)
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
                    Pumped::Notification(notification) => self.inbox.push(notification),
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

    fn position_encoding(&self) -> Encoding {
        Encoding::negotiated(
            self.server_capabilities()
                .and_then(|capabilities| capabilities.position_encoding.as_ref()),
        )
    }

    pub fn text_document_sync_kind(&self) -> Option<TextDocumentSyncKind> {
        self.server_capabilities()
            .and_then(core::capabilities::text_document_sync_kind)
    }

    pub fn supports_incremental_sync(&self) -> bool {
        self.text_document_sync_kind() == Some(TextDocumentSyncKind::Incremental)
    }

    pub fn supports_formatting(&self) -> bool {
        self.server_capabilities()
            .is_some_and(core::capabilities::supports_formatting)
    }

    pub fn supports_code_action(&self) -> bool {
        self.server_capabilities()
            .is_some_and(core::capabilities::supports_code_action)
    }

    pub fn supports_pull_diagnostics(&self) -> bool {
        self.server_capabilities()
            .is_some_and(core::capabilities::supports_pull_diagnostics)
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
                Pumped::Notification(notification) => self.inbox.push(notification),
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
                match message {
                    Message::Notification(notification) => Pumped::Notification(notification),
                    Message::Request(_) | Message::Response(_) => Pumped::Other,
                }
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

    /// A judge for pushed diagnostics for `uri` at its current version.
    fn diagnostics_judge(
        &self,
        uri: &Uri,
    ) -> crate::Result<impl FnMut(&Notification) -> Verdict<Vec<Diagnostic>> + use<>> {
        self.state.require_initialized()?;
        let version = self.docs.current_version(uri)?;
        let uri = uri.clone();
        Ok(move |notification: &Notification| {
            core::diagnostics::publication(&uri, version, notification)
        })
    }

    /// Judges notifications received earlier, then new ones as they arrive,
    /// until `judge` takes one or `deadline` passes (`None`). Notifications
    /// it keeps stay for later waits.
    fn wait_for_verdict<T>(
        &mut self,
        judge: &mut impl FnMut(&Notification) -> Verdict<T>,
        deadline: Instant,
        method: &'static str,
    ) -> crate::Result<Option<T>> {
        if let Some(taken) = self.inbox.take_first(&mut *judge) {
            return Ok(Some(taken));
        }
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            match self.pump(Some(deadline), None) {
                Pumped::Notification(notification) => match judge(&notification) {
                    Verdict::Take(taken) => return Ok(Some(taken)),
                    Verdict::Keep => self.inbox.push(notification),
                    Verdict::Discard => {}
                },
                Pumped::Closed(closed) => return Err(closed.into_error(method)),
                Pumped::Resolved(_) | Pumped::Other | Pumped::Quiet => {}
            }
        }
    }

    /// Moves everything already received into the inbox without waiting.
    fn stash_ready(&mut self) {
        loop {
            match self.pump(Some(Instant::now()), None) {
                Pumped::Notification(notification) => self.inbox.push(notification),
                Pumped::Resolved(_) | Pumped::Other => {}
                Pumped::Closed(_) | Pumped::Quiet => return,
            }
        }
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

fn whole_document(text: String) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangeWholeDocument { text }.into()
}

fn diagnostics_timeout(uri: &Uri, start: Instant) -> Error {
    ErrorKind::DiagnosticsTimeout {
        uri: uri.clone(),
        after_ms: elapsed_ms(start, Instant::now()),
    }
    .into()
}

fn parse<N: LspNotification>(notification: &Notification) -> Option<N::Params> {
    (notification.method == N::METHOD.as_str())
        .then(|| serde_json::from_value(notification.params.clone()).ok())
        .flatten()
}

enum Pumped {
    Notification(Notification),
    Resolved(crate::Result<serde_json::Value>),
    Closed(Closed),
    /// A server request (already answered) or a response for no one.
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
