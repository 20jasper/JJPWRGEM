pub(crate) mod capabilities;
pub(crate) mod transport;

use capabilities::initialize_params;

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

use std::{
    process::Command,
    time::{Duration, Instant},
};

use gen_lsp_types::{
    ClientCapabilities, CodeLensRefreshRequest, DiagnosticRefreshRequest, ExitNotification,
    FoldingRangeRefreshRequest, InitializeRequest, InitializeResult, InlayHintRefreshRequest,
    InlineValueRefreshRequest, MessageDirection, Notification, RegistrationRequest, Request,
    SemanticTokensRefreshRequest, ServerCapabilities, ShowMessageRequest, ShutdownRequest,
    TextDocumentContentRefreshRequest, UnregistrationRequest, WorkDoneProgressCreateRequest,
    WorkspaceFoldersRequest,
};
use lsp_server::{Message, Notification as RpcNotification, Request as RpcRequest, Response};
use serde_json::Value;

use self::transport::Transport;
use crate::error::{DisconnectKind, Error, ErrorKind, Result, elapsed_ms};

pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

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

fn to_value(params: impl serde::Serialize, method: &str) -> Result<Value> {
    serde_json::to_value(params).map_err(|e| Error::serialize_error(e, method))
}

fn from_value<T: serde::de::DeserializeOwned>(value: Value, method: &str) -> Result<T> {
    // Deserialize by reference so the success path (presumably the common
    // one) doesn't pay for a clone that's only needed if this fails --
    // `value` is still owned and available afterward either way.
    T::deserialize(&value).map_err(|source| {
        trace::log_deser_error(method, &value, &source);
        Error::deserialize_error(source, method, value)
    })
}

/// The client's own session lifecycle -- separate from any per-document
/// state a later feature slice adds. `shutdown()` is reachable from either
/// state (a server can be asked to shut down before it was ever
/// initialized; the server is expected to reject that itself, not
/// something lspresso blocks locally), so `ShutdownRequested` carries
/// whatever capabilities were on hand, if any, rather than requiring them.
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

/// A request's tracked state between `send_request` and `recv_response`.
/// Kept as one enum per id (rather than an "is it outstanding" set and a
/// separately-populated "buffered responses" map) so a response arriving
/// for an id that's already `Buffered` -- a duplicate the server sent
/// before the first copy was ever consumed -- is a state transition this
/// makes explicit, not a silent second `HashMap::insert` overwriting the
/// first with no error.
#[derive(Debug)]
pub(super) enum RequestState {
    Outstanding,
    Buffered(lsp_server::Response),
}

#[derive(Debug)]
pub struct Client {
    transport: Transport,
    next_id: i32,
    timeout: Duration,
    peak_rss_bytes: Option<u64>,
    lifecycle: Lifecycle,
    server_name: String,
    /// The state of every request sent but not yet matched by
    /// `await_response`. Needed once more than one request can be
    /// in-flight at a time: a response not matching the id currently being
    /// awaited isn't a stray message to drop -- it's owed to a later
    /// `await_response` call, unless its id isn't here at all or is
    /// already `Buffered`, in which case it's a protocol violation on the
    /// server's part (an id lspresso never sent, or a duplicate).
    requests: std::collections::HashMap<lsp_server::RequestId, RequestState>,
}

// --- Constructor -----------------------------------------------------------
impl Client {
    #[must_use]
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
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
            peak_rss_bytes: None,
            lifecycle: Lifecycle::Uninitialized,
            server_name,
            requests: std::collections::HashMap::new(),
        }
    }
}

// --- Lifecycle ------------------------------------------------------------
impl Client {
    // lspclient[impl init-initialized-after-response]
    pub fn initialize(&mut self, capabilities: ClientCapabilities) -> Result<InitializeResult> {
        self.initialize_hook(initialize_params(capabilities))
    }

    // Can't require_initialized() first -- sending Initialize is what makes
    // the client initialized -- so this records the server's capabilities
    // and sends Initialized itself instead.
    fn initialize_hook(
        &mut self,
        params: gen_lsp_types::InitializeParams,
    ) -> Result<InitializeResult> {
        if self.lifecycle.capabilities().is_some() {
            return Err(ErrorKind::AlreadyInitialized.into());
        }
        let result = self.request::<InitializeRequest>(params)?;
        // Confirm `initialized` actually sent before flipping lifecycle --
        // otherwise a transport failure here would leave the client marked
        // Initialized (satisfying require_initialized() everywhere else)
        // while the server never received it, with no way to retry since
        // the guard above would immediately reject a second attempt.
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

    /// Polls until the server process exits or `self.timeout` passes.
    /// Returns its exit code, or `None` if it hasn't exited within the
    /// timeout, or the OS couldn't report a code (e.g. killed by signal).
    // lspclient[impl shutdown-exit-terminates]
    // lspclient[impl shutdown-exit-code-reflects-shutdown]
    pub fn wait_for_exit(&mut self) -> Option<i32> {
        let deadline = Instant::now() + self.timeout;
        self.transport.wait_for_exit(deadline)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.timeout = Duration::from_millis(500);
        drop(self.shutdown());
    }
}

// --- Metrics --------------------------------------------------------------
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

// --- Message IO -----------------------------------------------------------
impl Client {
    // Every notification/request is written to the transport synchronously,
    // before the call that sent it returns.
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
        if let Some(resp) = self.take_buffered_response(&request_id) {
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
                    self.requests.remove(&request_id);
                    return Self::response_to_result(resp, method, id);
                }
                _ => self.handle_background(msg)?,
            }
        }
    }

    /// Removes and returns `id`'s response if one is already buffered,
    /// leaving anything else (an `Outstanding` entry, or no entry at all)
    /// untouched.
    fn take_buffered_response(
        &mut self,
        id: &lsp_server::RequestId,
    ) -> Option<lsp_server::Response> {
        if matches!(self.requests.get(id), Some(RequestState::Buffered(_))) {
            let Some(RequestState::Buffered(resp)) = self.requests.remove(id) else {
                unreachable!("checked Buffered above");
            };
            return Some(resp);
        }
        None
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
            // Any notification method is tolerated uniformly here, whether
            // it's one lspresso specifically acts on or not -- $/logTrace
            // and telemetry/event are purely informational and just need to
            // not cause an error.
            Message::Notification(note) => {
                tracing::trace!(method = %note.method, "dropped unsolicited notification");
            }
            Message::Request(req) => self.answer_server_request(req)?,
            Message::Response(resp) => {
                // Not necessarily stray: with more than one request in
                // flight, this can be owed to an `await_response` call for
                // a different id that hasn't happened yet. But if its id
                // was never sent, or already has a response buffered (a
                // duplicate, whether or not the first was ever consumed),
                // that's a protocol violation on the server's part -- error
                // instead of silently dropping or overwriting it.
                let is_outstanding =
                    matches!(self.requests.get(&resp.id), Some(RequestState::Outstanding));
                if is_outstanding {
                    self.requests
                        .insert(resp.id.clone(), RequestState::Buffered(resp));
                } else {
                    return Err(ErrorKind::UnmatchedResponseId {
                        id: resp.id.to_string(),
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    fn answer_server_request(&mut self, req: RpcRequest) -> Result<()> {
        // Expands `req.method == "a" || req.method == "b" || ...` for the
        // listed types, each guarded by a compile-time assertion that it's
        // actually declared `MessageDirection::ServerToClient` in
        // gen_lsp_types -- catches a typo'd or wrong-direction type landing
        // in the list, rather than silently miscompiling the check.
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
            // `null` is a spec-valid response for all of these:
            // showMessageRequest's result is `MessageActionItem | null`
            // (null meaning no action selected), `workspace/workspaceFolders`
            // is `WorkspaceFolder[] | null`, and workDoneProgress/create,
            // client/registerCapability, client/unregisterCapability, and
            // every `workspace/*/refresh` request (semanticTokens,
            // diagnostic, codeLens, foldingRange, inlayHint, inlineValue,
            // textDocumentContent) have a `void` result -- answering null
            // doesn't itself trigger recomputation, that's on whatever
            // caller decides to re-issue the underlying pull afterward.
            Response::new_ok(req.id, Value::Null)
        } else {
            // Every other `MessageDirection::ServerToClient` request
            // (including `workspace/configuration`, `workspace/applyEdit`,
            // and `window/showDocument`, not yet special-cased here) is a
            // method lspresso doesn't recognize at all -- per spec,
            // `$/`-prefixed or not, that's answered with an error, not a
            // silent success.
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

// --- Notification Wait ----------------------------------------------------
impl Client {
    /// Waits for a notification of type `N` matching `predicate`, tolerating
    /// (and dropping) anything else that arrives in the meantime.
    pub fn wait_for<N, F>(&mut self, mut predicate: F) -> Result<N::Params>
    where
        N: Notification,
        F: FnMut(&N::Params) -> bool,
    {
        let start = Instant::now();
        let deadline = Instant::now() + self.timeout;
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
                        Ok(_) => {
                            tracing::trace!(
                                method = N::METHOD.as_str(),
                                "dropped non-matching notification"
                            );
                        }
                    }
                }
                _ => self.handle_background(msg)?,
            }
        }
    }
}

// --- Request / Notify Plumbing -------------------------------------------
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
        self.requests.insert(id.into(), RequestState::Outstanding);
        Ok(id)
    }

    pub fn request<R: Request>(&mut self, params: R::Params) -> Result<R::Result> {
        let deadline = Instant::now() + self.timeout;
        let id =
            self.send_request_value(R::METHOD.as_str(), to_value(params, R::METHOD.as_str())?)?;
        let value = self.await_response(id, R::METHOD.as_str(), deadline)?;
        from_value(value, R::METHOD.as_str())
    }

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
