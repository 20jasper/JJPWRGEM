use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use gen_lsp_types::*;
use lsp_server::{Message, RequestId};

use super::Effect;
use crate::error::{ErrorKind, deadline_after, elapsed_ms};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    NotStarted,
    Initializing,
    Initialized,
    ShuttingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestKind {
    Initialize,
    Shutdown,
    Generic { method: &'static str },
}

impl RequestKind {
    const fn method_name(self) -> &'static str {
        match self {
            Self::Initialize => InitializeRequest::METHOD.as_str(),
            Self::Shutdown => ShutdownRequest::METHOD.as_str(),
            Self::Generic { method } => method,
        }
    }

    /// The lifecycle to roll back to on failure/timeout, or `None` if
    /// irrelevant to lifecycle
    const fn rollback_to(self) -> Option<Lifecycle> {
        match self {
            Self::Initialize => Some(Lifecycle::NotStarted),
            Self::Shutdown => Some(Lifecycle::Initialized),
            Self::Generic { .. } => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Pending {
    kind: RequestKind,
    started_at: Instant,
    deadline: Instant,
    /// avoid double processing and emitting duplicate failure effects
    timed_out: bool,
}

impl Pending {
    fn new(kind: RequestKind, started_at: Instant, timeout: Duration) -> Self {
        Self {
            kind,
            started_at,
            deadline: deadline_after(started_at, timeout),
            timed_out: false,
        }
    }
}

/// tracks pending requests, lifecycle, and id counts
#[derive(Debug, Clone)]
pub struct ClientState {
    lifecycle: Lifecycle,
    pending: BTreeMap<RequestId, Pending>,
    next_id: i32,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            lifecycle: Lifecycle::NotStarted,
            pending: BTreeMap::new(),
            next_id: 0,
        }
    }
}

impl ClientState {
    #[cfg_attr(not(test), expect(dead_code, reason = "only used by tests for now"))]
    fn lifecycle(&self) -> &Lifecycle {
        &self.lifecycle
    }

    pub fn require_initialized(&self) -> crate::Result<()> {
        if self.lifecycle == Lifecycle::Initialized {
            Ok(())
        } else {
            Err(ErrorKind::NotInitialized.into())
        }
    }

    /// takes in any params
    ///
    /// allocates id, tracks the request and builds the outgoing message
    fn request_raw(
        &mut self,
        kind: RequestKind,
        params: serde_json::Value,
        now: Instant,
        timeout: Duration,
    ) -> (lsp_server::RequestId, lsp_server::Message) {
        self.next_id += 1;

        let request_id = RequestId::from(self.next_id);
        self.pending
            .insert(request_id.clone(), Pending::new(kind, now, timeout));
        let message = Message::Request(lsp_server::Request::new(
            request_id.clone(),
            kind.method_name().to_owned(),
            params,
        ));
        (request_id, message)
    }

    pub fn start_initialize(
        &mut self,
        params: serde_json::Value,
        now: Instant,
        timeout: Duration,
    ) -> crate::Result<(RequestId, Message)> {
        if self.lifecycle != Lifecycle::NotStarted {
            return Err(ErrorKind::AlreadyInitialized.into());
        }
        let (id, message) = self.request_raw(RequestKind::Initialize, params, now, timeout);
        self.lifecycle = Lifecycle::Initializing;
        Ok((id, message))
    }

    pub fn start_generic(
        &mut self,
        method: &'static str,
        params: serde_json::Value,
        now: Instant,
        timeout: Duration,
    ) -> crate::Result<(RequestId, Message)> {
        self.require_initialized()?;
        Ok(self.request_raw(RequestKind::Generic { method }, params, now, timeout))
    }

    pub fn start_shutdown(
        &mut self,
        now: Instant,
        timeout: Duration,
    ) -> crate::Result<(RequestId, Message)> {
        match self.lifecycle {
            Lifecycle::NotStarted | Lifecycle::Initializing => {
                Err(ErrorKind::NotInitialized.into())
            }
            Lifecycle::ShuttingDown => Err(ErrorKind::AlreadyShutdown.into()),
            Lifecycle::Initialized => {
                let (id, message) =
                    self.request_raw(RequestKind::Shutdown, serde_json::Value::Null, now, timeout);
                self.lifecycle = Lifecycle::ShuttingDown;
                Ok((id, message))
            }
        }
    }

    pub fn rollback_initialize(&mut self) {
        if self.lifecycle == Lifecycle::Initialized {
            self.lifecycle = Lifecycle::NotStarted;
        }
    }

    pub fn cancel_pending_request(&mut self, id: &RequestId) {
        if let Some(pending) = self.pending.remove(id)
            && !pending.timed_out
            && let Some(lifecycle) = pending.kind.rollback_to()
        {
            self.lifecycle = lifecycle;
        }
    }

    pub fn on_message(&mut self, message: &Message) -> Vec<Effect> {
        let response = match message {
            Message::Response(response) => response,
            Message::Request(request) => {
                return vec![Effect::Reply(Message::Response(
                    super::server_requests::answer(request),
                ))];
            }
            Message::Notification(_) => return Vec::new(),
        };
        let Some(pending) = self.pending.remove(&response.id) else {
            return vec![Effect::Error(
                ErrorKind::UnmatchedResponseId {
                    id: response.id.clone(),
                }
                .into(),
            )];
        };
        if pending.timed_out {
            return Vec::new();
        }
        let result = match &response.response_result {
            Ok(result) => {
                if pending.kind == RequestKind::Initialize {
                    self.lifecycle = Lifecycle::Initialized;
                }
                Ok(result.clone())
            }
            Err(error) => {
                if let Some(lifecycle) = pending.kind.rollback_to() {
                    self.lifecycle = lifecycle;
                }
                Err(ErrorKind::Response {
                    code: error.code,
                    message: error.message.clone(),
                    method: pending.kind.method_name(),
                    id: response.id.clone(),
                    data: error.data.clone(),
                }
                .into())
            }
        };
        vec![Effect::Resolved(response.id.clone(), result)]
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.pending
            .values()
            .filter(|pending| !pending.timed_out)
            .map(|pending| pending.deadline)
            .min()
    }

    /// check for any past deadline requests and return them as failures
    pub fn expire_timed_out_requests(&mut self, now: Instant) -> Vec<Effect> {
        let (lifecycle_ref, pending_requests) = (&mut self.lifecycle, &mut self.pending);
        pending_requests
            .iter_mut()
            .filter_map(|(id, pending)| {
                if pending.timed_out || pending.deadline > now {
                    return None;
                }

                // timed out, roll back and return failure
                pending.timed_out = true;
                if let Some(lifecycle) = pending.kind.rollback_to() {
                    *lifecycle_ref = lifecycle;
                }
                Some(Effect::Resolved(
                    id.clone(),
                    Err(ErrorKind::ResponseTimeout {
                        method: pending.kind.method_name(),
                        id: id.clone(),
                        after_ms: elapsed_ms(pending.started_at, now),
                    }
                    .into()),
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::Error;

    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

    impl ClientState {
        #[cfg(test)]
        fn active_pending_kinds(&self) -> Vec<RequestKind> {
            self.pending
                .values()
                .filter(|pending| !pending.timed_out)
                .map(|pending| pending.kind)
                .collect()
        }
    }

    fn ok_response(id: RequestId, result: serde_json::Value) -> Message {
        Message::Response(lsp_server::Response::new_ok(id, result))
    }

    fn err_response(id: RequestId, code: i32, message: &str) -> Message {
        Message::Response(lsp_server::Response::new_err(id, code, message.to_owned()))
    }

    fn start_initialize_ok(state: &mut ClientState, now: Instant, timeout: Duration) -> RequestId {
        state
            .start_initialize(serde_json::json!({}), now, timeout)
            .unwrap()
            .0
    }

    fn start_shutdown_ok(state: &mut ClientState, now: Instant, timeout: Duration) -> RequestId {
        state.start_shutdown(now, timeout).unwrap().0
    }

    fn initialized(state: &mut ClientState, now: Instant) -> RequestId {
        let id = start_initialize_ok(state, now, DEFAULT_TIMEOUT);
        state.on_message(&ok_response(id.clone(), serde_json::json!(null)));
        id
    }

    fn single_effect(effects: Vec<Effect>) -> Effect {
        let [effect]: [Effect; 1] = effects.try_into().expect("exactly one effect");
        effect
    }

    fn resolved_result(
        effects: Vec<Effect>,
        expected_id: &RequestId,
    ) -> crate::Result<serde_json::Value> {
        let Effect::Resolved(id, result) = single_effect(effects) else {
            panic!("expected a Resolved effect");
        };
        assert_eq!(id, *expected_id);
        result
    }

    fn single_error(effects: Vec<Effect>) -> Error {
        let Effect::Error(err) = single_effect(effects) else {
            panic!("expected an Error effect");
        };
        err
    }

    #[test]
    fn start_initialize_then_response_moves_to_initialized() {
        let mut state = ClientState::default();
        let now = Instant::now();
        let id = start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);

        let effects = state.on_message(&ok_response(id.clone(), serde_json::json!(42)));

        assert_eq!(*state.lifecycle(), Lifecycle::Initialized);
        assert_eq!(
            resolved_result(effects, &id).unwrap(),
            serde_json::json!(42)
        );
    }

    #[test]
    fn second_initialize_is_rejected() {
        let mut state = ClientState::default();
        let now = Instant::now();
        start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);

        let err = state
            .start_initialize(serde_json::json!({}), now, DEFAULT_TIMEOUT)
            .unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::AlreadyInitialized));
    }

    #[test]
    fn shutdown_before_initialize_is_rejected() {
        let mut state = ClientState::default();
        let err = state
            .start_shutdown(Instant::now(), DEFAULT_TIMEOUT)
            .unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::NotInitialized));
    }

    #[test]
    fn require_initialized_rejects_everything_but_the_initialized_state() {
        let mut state = ClientState::default();
        assert!(state.require_initialized().is_err());

        let id = start_initialize_ok(&mut state, Instant::now(), DEFAULT_TIMEOUT);
        assert!(state.require_initialized().is_err());

        state.on_message(&ok_response(id, serde_json::json!(null)));
        assert!(state.require_initialized().is_ok());
    }

    #[test]
    fn error_response_rolls_lifecycle_back_and_allows_retry() {
        let mut state = ClientState::default();
        let now = Instant::now();
        let id = start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);

        let effects = state.on_message(&err_response(id.clone(), -32603, "boom"));

        assert_eq!(*state.lifecycle(), Lifecycle::NotStarted);
        let err = resolved_result(effects, &id).unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::Response { .. }));

        start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);
    }

    #[test]
    fn generic_requests_require_initialized() {
        let mut state = ClientState::default();
        let err = state
            .start_generic(
                "textDocument/formatting",
                serde_json::json!({}),
                Instant::now(),
                DEFAULT_TIMEOUT,
            )
            .unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::NotInitialized));
    }

    #[test]
    fn generic_request_failure_leaves_lifecycle_initialized_and_allows_retry() {
        let mut state = ClientState::default();
        let now = Instant::now();
        initialized(&mut state, now);

        let (id, _) = state
            .start_generic(
                "textDocument/formatting",
                serde_json::json!({}),
                now,
                DEFAULT_TIMEOUT,
            )
            .unwrap();
        state.on_message(&err_response(id, -32603, "boom"));
        assert_eq!(*state.lifecycle(), Lifecycle::Initialized);

        state
            .start_generic(
                "textDocument/formatting",
                serde_json::json!({}),
                now,
                DEFAULT_TIMEOUT,
            )
            .unwrap();
    }

    #[test]
    fn unmatched_response_id_is_reported_without_a_pending_match() {
        let mut state = ClientState::default();

        let effects = state.on_message(&ok_response(RequestId::from(999), serde_json::json!(null)));

        let err = single_error(effects);
        assert!(matches!(err.kind(), ErrorKind::UnmatchedResponseId { .. }));
    }

    #[test]
    fn duplicate_response_after_consumption_is_reported_as_unmatched() {
        let mut state = ClientState::default();
        let now = Instant::now();
        let id = start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);

        let first = state.on_message(&ok_response(id.clone(), serde_json::json!(null)));
        assert!(matches!(first.as_slice(), [Effect::Resolved(_, Ok(_))]));

        let duplicate = state.on_message(&ok_response(id, serde_json::json!(null)));
        let err = single_error(duplicate);
        assert!(matches!(err.kind(), ErrorKind::UnmatchedResponseId { .. }));
    }

    #[test]
    fn server_requests_are_answered_without_touching_state() {
        let mut state = ClientState::default();
        start_initialize_ok(&mut state, Instant::now(), DEFAULT_TIMEOUT);

        let request = lsp_server::Request::new(
            RequestId::from(1234),
            "workspace/configuration".to_owned(),
            serde_json::json!({ "items": [{}] }),
        );

        let Effect::Reply(Message::Response(reply)) =
            single_effect(state.on_message(&Message::Request(request.clone())))
        else {
            panic!("expected a Reply with a Response");
        };
        assert_eq!(reply.id, request.id);
        assert_eq!(reply.response_result.unwrap(), serde_json::json!([null]));
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);
        assert_eq!(state.active_pending_kinds(), vec![RequestKind::Initialize]);
    }

    #[test]
    fn notifications_are_ignored_by_on_message() {
        let mut state = ClientState::default();
        start_initialize_ok(&mut state, Instant::now(), DEFAULT_TIMEOUT);

        let notification = Message::Notification(lsp_server::Notification::new(
            "$/logTrace".to_owned(),
            serde_json::json!(null),
        ));

        assert!(state.on_message(&notification).is_empty());
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);
        assert_eq!(state.active_pending_kinds(), vec![RequestKind::Initialize]);
    }

    #[test]
    fn tick_exactly_at_the_deadline_times_out_one_tick_before_does_not() {
        let mut state = ClientState::default();
        let start = Instant::now();
        let id = start_initialize_ok(&mut state, start, Duration::from_millis(10));

        let one_tick_before = start + Duration::from_millis(9);
        assert!(state.expire_timed_out_requests(one_tick_before).is_empty());
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);

        let exactly_at_deadline = start + Duration::from_millis(10);
        let effects = state.expire_timed_out_requests(exactly_at_deadline);
        assert_eq!(*state.lifecycle(), Lifecycle::NotStarted);
        let err = resolved_result(effects, &id).unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));
    }

    #[test]
    fn timeouts_name_the_method_that_timed_out() {
        fn timed_out_method(state: &mut ClientState, id: &RequestId, now: Instant) -> &'static str {
            let err = resolved_result(state.expire_timed_out_requests(now), id).unwrap_err();
            let ErrorKind::ResponseTimeout { method, .. } = err.kind() else {
                panic!("expected a timeout: {err:?}");
            };
            method
        }
        let mut state = ClientState::default();
        let start = Instant::now();
        let later = start + Duration::from_millis(1);

        let id = start_initialize_ok(&mut state, start, Duration::ZERO);
        assert_eq!(timed_out_method(&mut state, &id, later), "initialize");

        initialized(&mut state, later);
        let (id, _) = state
            .start_generic(
                "textDocument/formatting",
                serde_json::json!({}),
                later,
                Duration::ZERO,
            )
            .unwrap();
        assert_eq!(
            timed_out_method(&mut state, &id, later),
            "textDocument/formatting"
        );

        let id = start_shutdown_ok(&mut state, later, Duration::ZERO);
        assert_eq!(timed_out_method(&mut state, &id, later), "shutdown");
    }

    #[test]
    fn the_next_deadline_is_the_earliest_one_not_yet_timed_out() {
        let mut state = ClientState::default();
        let start = Instant::now();
        assert_eq!(state.next_deadline(), None);

        initialized(&mut state, start);
        let (late, _) = state
            .start_generic(
                "a",
                serde_json::Value::Null,
                start,
                Duration::from_millis(30),
            )
            .unwrap();
        state
            .start_generic(
                "b",
                serde_json::Value::Null,
                start,
                Duration::from_millis(10),
            )
            .unwrap();
        assert_eq!(
            state.next_deadline(),
            Some(start + Duration::from_millis(10))
        );

        state.expire_timed_out_requests(start + Duration::from_millis(10));
        assert_eq!(
            state.next_deadline(),
            Some(start + Duration::from_millis(30))
        );

        state.cancel_pending_request(&late);
        assert_eq!(state.next_deadline(), None);
    }

    #[test]
    fn tick_past_deadline_times_out_and_allows_retry() {
        let mut state = ClientState::default();
        let start = Instant::now();
        let id = start_initialize_ok(&mut state, start, Duration::from_millis(10));

        let past_deadline = start + Duration::from_millis(11);
        let effects = state.expire_timed_out_requests(past_deadline);

        assert_eq!(*state.lifecycle(), Lifecycle::NotStarted);
        let err = resolved_result(effects, &id).unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::ResponseTimeout { .. }));

        start_initialize_ok(&mut state, past_deadline, DEFAULT_TIMEOUT);
    }

    #[test]
    fn late_response_after_timeout_is_ignored_not_reported_as_unmatched() {
        let mut state = ClientState::default();
        let start = Instant::now();
        let id = start_initialize_ok(&mut state, start, Duration::from_millis(10));

        let past_deadline = start + Duration::from_millis(11);
        let timeout_effects = state.expire_timed_out_requests(past_deadline);
        assert_eq!(timeout_effects.len(), 1);

        let effects = state.on_message(&ok_response(id, serde_json::json!("too late")));

        assert!(effects.is_empty());
    }

    #[test]
    fn shutdown_timeout_rolls_back_to_initialized_and_allows_retry() {
        let mut state = ClientState::default();
        let start = Instant::now();
        initialized(&mut state, start);
        assert_eq!(*state.lifecycle(), Lifecycle::Initialized);

        start_shutdown_ok(&mut state, start, Duration::from_millis(10));
        let past_deadline = start + Duration::from_millis(11);
        state.expire_timed_out_requests(past_deadline);

        assert_eq!(*state.lifecycle(), Lifecycle::Initialized);

        start_shutdown_ok(&mut state, past_deadline, DEFAULT_TIMEOUT);
    }

    #[test]
    fn cancel_rolls_back_a_never_sent_request_and_allows_retry() {
        let mut state = ClientState::default();
        let now = Instant::now();
        let id = start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);

        state.cancel_pending_request(&id);

        assert_eq!(*state.lifecycle(), Lifecycle::NotStarted);
        start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);
    }

    #[test]
    fn cancelling_an_already_timed_out_id_does_not_roll_back_a_newer_request() {
        let mut state = ClientState::default();
        let now = Instant::now();
        let stale_id = start_initialize_ok(&mut state, now, Duration::ZERO);
        state.expire_timed_out_requests(now);
        assert_eq!(*state.lifecycle(), Lifecycle::NotStarted);

        let current_id = start_initialize_ok(&mut state, now, DEFAULT_TIMEOUT);
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);

        // `stale_id` already resolved via timeout. cancelling it must not roll
        // back the unrelated requests
        state.cancel_pending_request(&stale_id);
        assert_eq!(*state.lifecycle(), Lifecycle::Initializing);

        state.on_message(&ok_response(current_id, serde_json::json!(null)));
        assert_eq!(*state.lifecycle(), Lifecycle::Initialized);
    }

    #[test]
    fn cancel_of_an_unknown_id_is_a_no_op() {
        let mut state = ClientState::default();
        state.cancel_pending_request(&RequestId::from(999));
        assert_eq!(*state.lifecycle(), Lifecycle::NotStarted);
    }
}
