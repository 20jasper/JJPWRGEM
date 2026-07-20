pub(crate) mod capabilities;
mod diagnostics;
pub(crate) mod transport;
mod with_uri;

use capabilities::initialize_params;
use with_uri::WithUri;

mod trace {
    use lsp_server::Message;
    use serde_json::Value;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Direction {
        Send,
        Recv,
    }

    impl Direction {
        fn as_str(self) -> &'static str {
            match self {
                Self::Send => "send",
                Self::Recv => "receive",
            }
        }
    }

    pub(super) fn log_rpc(direction: Direction, message: &Message) {
        let direction = direction.as_str();
        if let Some(method) = method_name(message) {
            tracing::debug!(direction, method, summary = %summarize(message), "rpc");
            tracing::trace!(direction, method, raw = %params_json(message), "rpc");
        } else {
            tracing::debug!(direction, summary = %summarize(message), "rpc");
            tracing::trace!(direction, raw = %params_json(message), "rpc");
        }
    }

    pub(super) fn log_deser_error(method: &str, raw: &Value, error: &serde_json::Error) {
        tracing::warn!(method, summary = %brief(raw), error = %error, "deserialization failed");
        tracing::trace!(method, raw = %raw, "deserialization failed");
    }

    fn method_name(message: &Message) -> Option<&str> {
        match message {
            Message::Request(r) => Some(&r.method),
            Message::Notification(n) => Some(&n.method),
            Message::Response(_) => None,
        }
    }

    fn summarize(message: &Message) -> String {
        match message {
            Message::Request(request) => {
                format!(
                    "request#{} {} {}",
                    request.id,
                    request.method,
                    brief(&request.params)
                )
            }
            Message::Response(response) => match &response.response_kind {
                lsp_server::ResponseKind::Err { error } => format!(
                    "response#{} error {}: {}",
                    response.id, error.code, error.message
                ),
                lsp_server::ResponseKind::Ok { result } => {
                    format!("response#{} {}", response.id, brief(result))
                }
            },
            Message::Notification(notification) => {
                format!(
                    "notification {} {}",
                    notification.method,
                    brief(&notification.params)
                )
            }
        }
    }

    fn params_json(message: &Message) -> String {
        match message {
            Message::Request(r) => r.params.to_string(),
            Message::Response(r) => match &r.response_kind {
                lsp_server::ResponseKind::Err { error } => {
                    serde_json::json!({"code": error.code, "message": error.message}).to_string()
                }
                lsp_server::ResponseKind::Ok { result } => result.to_string(),
            },
            Message::Notification(n) => n.params.to_string(),
        }
    }

    fn brief(value: &Value) -> String {
        const MAX: usize = 400;
        let text = value.to_string();
        let mut chars = text.chars();
        let head: String = chars.by_ref().take(MAX).collect();
        if chars.next().is_some() {
            format!("{head}…")
        } else {
            text
        }
    }
}

// lspclient[impl uri-identifies-document]
// lspclient[impl uri-consistent]
// Uri equality is exact string match, not normalized -- lspresso doesn't
// paper over a caller using inconsistent representations of "the same"
// document, since doing so would require guessing which representation
// the server also considers canonical.
mod open_docs {
    use std::collections::HashSet;

    use gen_lsp_types::Uri;

    use crate::error::{ErrorKind, Result};

    #[derive(Debug, Default)]
    pub(super) struct OpenDocs(HashSet<Uri>);

    impl OpenDocs {
        // lspclient[impl didopen-max-one-open]
        // lspclient[impl didopen-close-open-balanced]
        pub(super) fn require_not_open(&self, uri: &Uri) -> Result<()> {
            if self.0.contains(uri) {
                return Err(ErrorKind::AlreadyOpen { uri: uri.clone() }.into());
            }
            Ok(())
        }

        // lspclient[impl lifecycle-didchange-only-when-open]
        pub(super) fn require_open(&self, uri: &Uri) -> Result<()> {
            if self.0.contains(uri) {
                Ok(())
            } else {
                Err(ErrorKind::NotOpen { uri: uri.clone() }.into())
            }
        }

        pub(super) fn mark_open(&mut self, uri: Uri) {
            self.0.insert(uri);
        }

        pub(super) fn close(&mut self, uri: &Uri) {
            self.0.remove(uri);
        }
    }
}

mod notification_buf {
    use gen_lsp_types::{
        Notification, PublishDiagnosticsNotification, PublishDiagnosticsParams, Uri,
    };
    use lsp_server::Notification as RpcNotification;

    #[derive(Debug, Default)]
    pub(super) struct NotificationBuffer {
        buf: Vec<RpcNotification>,
    }

    impl NotificationBuffer {
        pub(super) fn push(&mut self, note: RpcNotification) {
            self.buf.push(note);
        }

        pub(super) fn take_first_where<N, F>(&mut self, predicate: &mut F) -> Option<N::Params>
        where
            N: Notification,
            F: FnMut(&N::Params) -> bool,
        {
            self.buf.retain(|note| {
                if note.method != N::METHOD.as_str() {
                    return true;
                }
                match serde_json::from_value::<N::Params>(note.params.clone()) {
                    Ok(_) => true,
                    Err(e) => {
                        tracing::warn!(method = N::METHOD.as_str(), error = %e, "dropped malformed notification from buffer");
                        false
                    }
                }
            });

            let idx = self.buf.iter().position(|n| {
                n.method == N::METHOD.as_str()
                    && predicate(
                        &serde_json::from_value::<N::Params>(n.params.clone())
                            .expect("malformed entries already filtered above"),
                    )
            })?;

            let note = self.buf.remove(idx);
            Some(
                serde_json::from_value::<N::Params>(note.params)
                    .expect("malformed entries already filtered above"),
            )
        }

        pub(super) fn drain_diagnostics(&mut self, uri: &Uri) {
            self.buf.retain(|n| {
                if n.method != PublishDiagnosticsNotification::METHOD.as_str() {
                    return true;
                }
                match serde_json::from_value::<PublishDiagnosticsParams>(n.params.clone()) {
                    Ok(p) => p.uri != *uri,
                    Err(_) => true,
                }
            });
        }
    }
}

use std::{
    collections::HashMap,
    process::Command,
    time::{Duration, Instant},
};

use gen_lsp_types::{
    ClientCapabilities, CodeLensRefreshRequest, Diagnostic, DiagnosticRefreshRequest,
    DidChangeTextDocumentNotification, DidCloseTextDocumentNotification,
    DidCloseTextDocumentParams, DidOpenTextDocumentNotification, DidOpenTextDocumentParams,
    ExitNotification, FoldingRangeRefreshRequest, InitializeRequest, InitializeResult,
    InlayHintRefreshRequest, InlineValueRefreshRequest, MessageDirection, Notification,
    RegistrationRequest, Request, SemanticTokensRefreshRequest, ServerCapabilities,
    SetTraceNotification, ShowMessageRequest, ShutdownRequest, TextDocumentContentRefreshRequest,
    TextDocumentIdentifier, UnregistrationRequest, Uri, WorkDoneProgressCreateRequest,
    WorkspaceFoldersRequest,
};
use lsp_server::{Message, Notification as RpcNotification, Request as RpcRequest, Response};
use notification_buf::NotificationBuffer;
use serde_json::Value;

use self::transport::Transport;
use crate::error::{DisconnectKind, Error, ErrorKind, Result, elapsed_ms};

pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_SETTLE: Duration = Duration::from_millis(50);

fn default_duration(env_var: &str, parse: fn(u64) -> Duration, fallback: Duration) -> Duration {
    match std::env::var(env_var) {
        Err(_) => fallback,
        Ok(raw) => match raw.parse() {
            Ok(val) => parse(val),
            Err(e) => {
                tracing::warn!(env_var, raw, error = %e, "not a valid u64; using the default");
                fallback
            }
        },
    }
}

fn default_timeout() -> Duration {
    default_duration(
        "LSPRESSO_DEFAULT_TIMEOUT_SECS",
        Duration::from_secs,
        DEFAULT_TIMEOUT,
    )
}

fn default_settle() -> Duration {
    default_duration(
        "LSPRESSO_DEFAULT_SETTLE_MS",
        Duration::from_millis,
        DEFAULT_SETTLE,
    )
}

fn text_document(uri: Uri) -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri }
}

fn to_value(params: impl serde::Serialize, method: &str) -> Result<Value> {
    serde_json::to_value(params).map_err(|e| Error::serialize_error(e, method))
}

fn from_value<T: serde::de::DeserializeOwned>(value: Value, method: &str) -> Result<T> {
    T::deserialize(&value).map_err(|source| {
        trace::log_deser_error(method, &value, &source);
        Error::deserialize_error(source, method, value)
    })
}

macro_rules! assert_client_to_server {
    ($item:ty, $trait:ident) => {
        const {
            assert!(matches!(
                <$item as $trait>::MESSAGE_DIRECTION,
                MessageDirection::ClientToServer | MessageDirection::Both
            ));
        }
    };
}

macro_rules! lsp_request {
    ($name:ident, $request:path $(, pre: $pre:ident)?) => {
        pub fn $name(
            &mut self,
            params: <$request as Request>::Params,
        ) -> Result<<$request as Request>::Result> {
            assert_client_to_server!($request, Request);
            self.require_initialized()?;
            $( self.$pre(&params)?; )?
            self.request::<$request>(params)
        }
    };
}

macro_rules! lsp_notify {
    ($name:ident, $notification:path $(, pre: $pre:ident $(, post: $post:ident)?)?) => {
        pub fn $name(&mut self, params: <$notification as Notification>::Params) -> Result<()> {
            assert_client_to_server!($notification, Notification);
            self.require_initialized()?;
            $( self.$pre(&params)?; )?
            self.notify::<$notification>(&params)?;
            $( $( self.$post(params)?; )? )?
            Ok(())
        }
    };
}

#[derive(Debug)]
enum Lifecycle {
    Uninitialized,
    Initialized(ServerCapabilities),
    ShutdownRequested(Option<ServerCapabilities>),
}

impl Lifecycle {
    fn capabilities(&self) -> Option<&ServerCapabilities> {
        match self {
            Lifecycle::Uninitialized => None,
            Lifecycle::Initialized(caps) => Some(caps),
            Lifecycle::ShutdownRequested(caps) => caps.as_ref(),
        }
    }

    fn is_shutdown(&self) -> bool {
        matches!(self, Lifecycle::ShutdownRequested(_))
    }
}

#[derive(Debug)]
enum RequestState {
    Outstanding,
    Buffered(lsp_server::Response),
}

#[derive(Debug, Default)]
struct PendingRequests(std::collections::HashMap<lsp_server::RequestId, RequestState>);

impl PendingRequests {
    fn mark_outstanding(&mut self, id: lsp_server::RequestId) {
        self.0.insert(id, RequestState::Outstanding);
    }

    fn resolve(&mut self, id: &lsp_server::RequestId) {
        self.0.remove(id);
    }

    /// Takes `id`'s buffered response if present, leaving anything else
    /// (an `Outstanding` entry, or no entry at all) untouched.
    fn take_buffered(&mut self, id: &lsp_server::RequestId) -> Option<lsp_server::Response> {
        match self.0.remove(id) {
            Some(RequestState::Buffered(resp)) => Some(resp),
            Some(other) => {
                self.0.insert(id.clone(), other);
                None
            }
            None => None,
        }
    }

    /// Buffers `resp` if its id is outstanding. Returns `false` for an id
    /// lspresso never sent or that already has a buffered response -- a
    /// protocol violation on the server's part, for the caller to report.
    fn try_buffer(&mut self, resp: lsp_server::Response) -> bool {
        if matches!(self.0.get(&resp.id), Some(RequestState::Outstanding)) {
            self.0.insert(resp.id.clone(), RequestState::Buffered(resp));
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct Client {
    transport: Transport,
    next_id: i32,
    timeout: Duration,
    settle: Duration,
    peak_rss_bytes: Option<u64>,
    lifecycle: Lifecycle,
    server_name: String,
    requests: PendingRequests,
    open_docs: open_docs::OpenDocs,
    notifications: NotificationBuffer,
    diagnostic_cache: HashMap<Uri, Vec<Diagnostic>>,
    diagnostic_result_ids: HashMap<Uri, String>,
}

impl Client {
    #[must_use]
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub fn with_settle(mut self, settle: Duration) -> Self {
        self.settle = settle;
        self
    }

    pub fn spawn_initialized(cmd: Command, capabilities: ClientCapabilities) -> Result<Self> {
        let mut client = Self::spawn(cmd)?;
        client.initialize(capabilities)?;
        Ok(client)
    }

    pub fn spawn(cmd: Command) -> Result<Self> {
        let server_name = {
            let p = cmd.get_program();
            std::path::Path::new(p)
                .file_name()
                .unwrap_or(p)
                .to_string_lossy()
                .into_owned()
        };
        Ok(Self::new_with_transport(
            Transport::spawn(cmd)?,
            server_name,
        ))
    }

    fn new_with_transport(transport: Transport, server_name: String) -> Self {
        Self {
            transport,
            next_id: 0,
            timeout: default_timeout(),
            settle: default_settle(),
            peak_rss_bytes: None,
            lifecycle: Lifecycle::Uninitialized,
            server_name,
            requests: PendingRequests::default(),
            open_docs: open_docs::OpenDocs::default(),
            notifications: NotificationBuffer::default(),
            diagnostic_cache: HashMap::new(),
            diagnostic_result_ids: HashMap::new(),
        }
    }
}

impl Client {
    pub fn initialize(&mut self, capabilities: ClientCapabilities) -> Result<InitializeResult> {
        self.protocol_initialize_request(initialize_params(capabilities))
    }

    // lspclient[impl init-initialized-after-response]
    //
    // Can't go through `lsp_request!`/`require_initialized` like every other
    // protocol method -- sending Initialize is what makes the client
    // initialized, so this hand-rolls the same `assert_client_to_server!`
    // check and does its own bookkeeping instead.
    pub fn protocol_initialize_request(
        &mut self,
        params: gen_lsp_types::InitializeParams,
    ) -> Result<InitializeResult> {
        assert_client_to_server!(InitializeRequest, Request);
        if self.lifecycle.capabilities().is_some() {
            return Err(ErrorKind::AlreadyInitialized.into());
        }
        let result = self.request::<InitializeRequest>(params)?;
        self.notify::<gen_lsp_types::InitializedNotification>(
            &gen_lsp_types::InitializedParams {},
        )?;
        self.lifecycle = Lifecycle::Initialized(result.capabilities.clone());
        tracing::info!(server = %self.server_name, "initialized");
        Ok(result)
    }

    // lspclient[impl shutdown-before-exit]
    pub fn shutdown(&mut self) -> Result<()> {
        if self.lifecycle.is_shutdown() {
            return Ok(());
        }
        self.request::<ShutdownRequest>(())?;
        self.notify::<ExitNotification>(&())?;
        self.lifecycle = Lifecycle::ShutdownRequested(self.lifecycle.capabilities().cloned());
        tracing::info!(server = %self.server_name, "shutdown");
        Ok(())
    }

    // lspclient[impl shutdown-exit-terminates]
    // lspclient[impl shutdown-exit-code-reflects-shutdown]
    pub fn wait_for_exit(&mut self) -> Option<i32> {
        let deadline = Instant::now() + self.timeout;
        self.transport.wait_for_exit(deadline)
    }

    // lspclient[impl init-first]
    fn require_initialized(&self) -> Result<()> {
        if self.lifecycle.capabilities().is_some() {
            Ok(())
        } else {
            Err(ErrorKind::NotInitialized.into())
        }
    }

    // lspclient[impl trace-client-may-set]
    lsp_notify!(protocol_set_trace_notification, SetTraceNotification);
    lsp_notify!(protocol_exit_notification, ExitNotification);
    lsp_request!(protocol_shutdown_request, ShutdownRequest);
}

impl Client {
    fn require_not_open<P: WithUri>(&self, params: &P) -> Result<()> {
        self.open_docs.require_not_open(params.uri())
    }

    fn require_open<P: WithUri>(&self, params: &P) -> Result<()> {
        self.open_docs.require_open(params.uri())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "every post: hook returns Result<()> so lsp_notify! can call them uniformly, even though this one can't actually fail"
    )]
    fn track_opened(&mut self, params: DidOpenTextDocumentParams) -> Result<()> {
        self.open_docs.mark_open(params.text_document.uri);
        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "every post: hook returns Result<()> so lsp_notify! can call them uniformly, even though this one can't actually fail"
    )]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "every post: hook takes owned Params so lsp_notify! can call them uniformly; this one just doesn't need to move anything out"
    )]
    fn track_closed(&mut self, params: DidCloseTextDocumentParams) -> Result<()> {
        self.open_docs.close(&params.text_document.uri);
        self.diagnostic_result_ids.remove(&params.text_document.uri);
        self.diagnostic_cache.remove(&params.text_document.uri);
        Ok(())
    }

    // lspclient[impl sync-mandatory]
    lsp_notify!(
        protocol_did_open_notification,
        DidOpenTextDocumentNotification,
        pre: require_not_open,
        post: track_opened
    );

    // lspclient[impl sync-mandatory]
    lsp_notify!(
        protocol_did_change_notification,
        DidChangeTextDocumentNotification,
        pre: require_open
    );

    // lspclient[impl sync-mandatory]
    // lspclient[impl didopen-close-open-balanced]
    lsp_notify!(
        protocol_did_close_notification,
        DidCloseTextDocumentNotification,
        pre: require_open,
        post: track_closed
    );
}

impl Drop for Client {
    fn drop(&mut self) {
        self.timeout = Duration::from_millis(500);
        drop(self.shutdown());
    }
}

impl Client {
    fn sample_rss(&mut self) {
        if let Some(bytes) = self.transport.current_rss_bytes() {
            self.peak_rss_bytes = Some(self.peak_rss_bytes.map_or(bytes, |peak| peak.max(bytes)));
        }
    }

    pub fn rss_bytes(&self) -> Option<u64> {
        self.peak_rss_bytes
    }
}

impl Client {
    fn send(&mut self, message: &Message) -> Result<()> {
        trace::log_rpc(trace::Direction::Send, message);
        let method = match message {
            Message::Request(r) => r.method.as_str(),
            Message::Notification(n) => n.method.as_str(),
            Message::Response(_) => "<response>",
        };
        self.transport.send(message).map_err(|e| match e {
            transport::Error::Io(e) => ErrorKind::Io(e).into(),
            transport::Error::Disconnected(code) => Error::disconnected(
                DisconnectKind::Send {
                    method: method.to_owned(),
                },
                code,
            ),
            transport::Error::Timeout => unreachable!("send cannot timeout"),
        })
    }

    fn response_to_result(resp: Response, method: &str, id: i32) -> Result<Value> {
        match resp.response_kind {
            lsp_server::ResponseKind::Err { error: err } => {
                // lspclient[impl errcode-server-not-initialized]
                // lspclient[impl errcode-content-modified]
                // lspclient[impl errcode-request-cancelled]
                Err(ErrorKind::Response {
                    code: err.code,
                    message: err.message,
                    method: method.to_owned(),
                    id,
                    data: err.data,
                }
                .into())
            }
            lsp_server::ResponseKind::Ok { result } => Ok(result),
        }
    }

    // lspclient[impl reqorder-response-matches-id]
    fn await_response(&mut self, id: i32, method: &str, deadline: Instant) -> Result<Value> {
        let request_id: lsp_server::RequestId = id.into();
        if let Some(resp) = self.requests.take_buffered(&request_id) {
            return Self::response_to_result(resp, method, id);
        }
        let start = Instant::now();
        loop {
            let msg = self.recv_until(
                deadline,
                || ErrorKind::ResponseTimeout {
                    method: method.to_owned(),
                    id,
                    after_ms: elapsed_ms(&start),
                },
                || DisconnectKind::Response {
                    method: method.to_owned(),
                    id,
                },
            )?;
            self.sample_rss();
            match msg {
                Message::Response(resp) if resp.id == request_id => {
                    self.requests.resolve(&request_id);
                    return Self::response_to_result(resp, method, id);
                }
                _ => self.handle_background(msg)?,
            }
        }
    }

    fn recv_until<F, G>(
        &mut self,
        deadline: Instant,
        on_timeout: F,
        on_disconnect: G,
    ) -> Result<Message>
    where
        F: FnOnce() -> ErrorKind,
        G: FnOnce() -> DisconnectKind,
    {
        let message = self
            .transport
            .recv_deadline(deadline)
            .map_err(|e| -> Error {
                match e {
                    transport::Error::Timeout => on_timeout().into(),
                    transport::Error::Disconnected(code) => {
                        Error::disconnected(on_disconnect(), code)
                    }
                    transport::Error::Io(e) => ErrorKind::Io(e).into(),
                }
            })?;
        trace::log_rpc(trace::Direction::Recv, &message);
        Ok(message)
    }

    fn handle_background(&mut self, msg: Message) -> Result<()> {
        match msg {
            // lspclient[impl trace-logtrace-tolerated]
            // lspclient[impl window-telemetry-tolerated]
            Message::Notification(note) => self.notifications.push(note),
            Message::Request(req) => self.answer_server_request(req)?,
            Message::Response(resp) => {
                let id = resp.id.clone();
                if !self.requests.try_buffer(resp) {
                    return Err(ErrorKind::UnmatchedResponseId { id: id.to_string() }.into());
                }
            }
        }
        Ok(())
    }

    fn answer_server_request(&mut self, req: RpcRequest) -> Result<()> {
        macro_rules! any_server_to_client_method {
            ($($t:ty),+ $(,)?) => {{
                $(
                    const _: () = assert!(matches!(
                        <$t as Request>::MESSAGE_DIRECTION,
                        MessageDirection::ServerToClient
                    ));
                )+
                $(req.method == <$t as Request>::METHOD.as_str())||+
            }};
        }

        let response = if any_server_to_client_method!(
            WorkspaceFoldersRequest,
            FoldingRangeRefreshRequest,
            WorkDoneProgressCreateRequest,
            SemanticTokensRefreshRequest,
            InlineValueRefreshRequest,
            InlayHintRefreshRequest,
            DiagnosticRefreshRequest,
            TextDocumentContentRefreshRequest,
            RegistrationRequest,
            UnregistrationRequest,
            ShowMessageRequest,
            CodeLensRefreshRequest,
        ) {
            // lspclient[impl window-showmessagerequest-responds]
            // lspclient[impl progress-create-acknowledged]
            // lspclient[impl refresh-client-recomputes]
            Response::new_ok(req.id, Value::Null)
        } else {
            // workspace/configuration, workspace/applyEdit, and window/showDocument
            // are deliberately unimplemented, not missed -- they fall here too.
            tracing::warn!(
                method = %req.method,
                "unrecognized server-initiated request; answering MethodNotFound"
            );
            Response::new_err(
                req.id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                format!("method not found: {}", req.method),
            )
        };
        self.send(&Message::Response(response))
    }
}

impl Client {
    pub fn wait_for<N, F>(&mut self, predicate: F) -> Result<N::Params>
    where
        N: Notification,
        F: FnMut(&N::Params) -> bool,
    {
        self.wait_for_within::<N, F>(self.timeout, predicate)
    }

    pub(super) fn wait_for_within<N, F>(
        &mut self,
        timeout: Duration,
        mut predicate: F,
    ) -> Result<N::Params>
    where
        N: Notification,
        F: FnMut(&N::Params) -> bool,
    {
        if let Some(params) = self.notifications.take_first_where::<N, _>(&mut predicate) {
            return Ok(params);
        }
        let start = Instant::now();
        let deadline = Instant::now() + timeout;
        loop {
            let msg = self.recv_until(
                deadline,
                || ErrorKind::NotificationTimeout {
                    method: N::METHOD.to_string(),
                    after_ms: elapsed_ms(&start),
                },
                || DisconnectKind::Notification {
                    method: N::METHOD.to_string(),
                },
            )?;
            self.sample_rss();
            match msg {
                Message::Notification(note) if note.method == N::METHOD.as_str() => {
                    match serde_json::from_value::<N::Params>(note.params.clone()) {
                        Err(e) => {
                            tracing::warn!(method = N::METHOD.as_str(), error = %e, "dropped malformed notification");
                        }
                        Ok(params) if predicate(&params) => return Ok(params),
                        Ok(_) => self.notifications.push(note),
                    }
                }
                _ => self.handle_background(msg)?,
            }
        }
    }
}

impl Client {
    fn check_not_shutdown(&self) -> Result<()> {
        if self.lifecycle.is_shutdown() {
            return Err(ErrorKind::AlreadyShutdown.into());
        }
        Ok(())
    }

    fn send_request_value(&mut self, method: &str, params: Value) -> Result<i32> {
        self.check_not_shutdown()?;
        let id = self.next_id.checked_add(1).expect("request ID exhausted");
        self.next_id = id;
        self.send(&Message::Request(RpcRequest::new(
            id.into(),
            method.to_owned(),
            params,
        )))?;
        self.requests.mark_outstanding(id.into());
        Ok(id)
    }

    fn request_value(&mut self, method: &str, params: Value) -> Result<Value> {
        let deadline = Instant::now() + self.timeout;
        self.request_value_before(method, params, deadline)
    }

    pub(super) fn request_value_before(
        &mut self,
        method: &str,
        params: Value,
        deadline: Instant,
    ) -> Result<Value> {
        let id = self.send_request_value(method, params)?;
        self.await_response(id, method, deadline)
    }

    /// Low-level escape hatch. Does not enforce invariants like document
    /// open or server initialization since it accepts arbitrary requests.
    pub fn request<R: Request>(&mut self, params: R::Params) -> Result<R::Result> {
        let value =
            self.request_value(R::METHOD.as_str(), to_value(params, R::METHOD.as_str())?)?;
        from_value(value, R::METHOD.as_str())
    }

    /// Low-level escape hatch. Does not enforce invariants like document
    /// open or server initialization since it accepts arbitrary requests.
    pub fn notify<N: Notification>(&mut self, params: &N::Params) -> Result<()> {
        self.notify_value(N::METHOD.as_str(), to_value(params, N::METHOD.as_str())?)
    }

    // lspclient[impl reqorder-notifications-no-response]
    fn notify_value(&mut self, method: &str, params: Value) -> Result<()> {
        self.check_not_shutdown()?;
        self.send(&Message::Notification(RpcNotification {
            method: method.to_owned(),
            params,
        }))
    }
}
