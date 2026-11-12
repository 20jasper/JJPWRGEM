use std::time::{Duration, Instant};

/// measures granular time spent receiving, processing, and responding to requests. Real
/// clients vary in speed so split out the language server's work from IO
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Timings {
    pub total: Duration,
    /// client time spent sending and serializing the message until the server has all bytes
    pub request: Duration,
    /// server time spent processing request until sending response
    pub server: Duration,
    /// client time spent receiving and deserializing response from the language server
    pub response: Duration,
}

impl Timings {
    #[must_use]
    pub const fn since(self, earlier: Self) -> Self {
        Self {
            total: self.total.saturating_sub(earlier.total),
            request: self.request.saturating_sub(earlier.request),
            server: self.server.saturating_sub(earlier.server),
            response: self.response.saturating_sub(earlier.response),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Stopwatch {
    timings: Timings,
    sent: u64,
    waiting_since: Option<Instant>,
    queued_at: Option<Instant>,
    written_at: Option<Instant>,
}

impl Stopwatch {
    pub(super) const fn timings(&self) -> Timings {
        self.timings
    }

    pub(super) fn sent(&mut self, started_at: Instant, queued_at: Instant) -> u64 {
        self.sent += 1;
        self.written_at = None;
        self.queued_at = Some(queued_at);
        self.waiting_since = Some(started_at);
        self.sent
    }

    pub(super) fn written(&mut self, sequence: u64, at: Instant) {
        if sequence == self.sent {
            self.written_at = Some(at);
        }
    }

    pub(super) fn arrived(&mut self, first_byte_at: Instant, parsed_at: Instant) {
        let Some(waiting_since) = self.waiting_since else {
            return;
        };
        let parsed_at = parsed_at.max(waiting_since);
        let first_byte_at = first_byte_at.clamp(waiting_since, parsed_at);
        let written_at = self
            .written_at
            .take()
            .or(self.queued_at.take())
            .map_or(waiting_since, |at| at.clamp(waiting_since, first_byte_at));
        self.timings.total += parsed_at - waiting_since;
        self.timings.request += written_at - waiting_since;
        self.timings.server += first_byte_at - written_at;
        self.timings.response += parsed_at - first_byte_at;
        self.waiting_since = Some(parsed_at);
    }

    pub(super) fn settle(&mut self, call_started_at: Instant) {
        let now = Instant::now();
        let from = self.waiting_since.map_or(call_started_at, |waiting_since| {
            waiting_since.max(call_started_at)
        });
        let decode = now.saturating_duration_since(from);
        self.timings.response += decode;
        self.timings.total += decode;
        if self.waiting_since.is_some() {
            self.waiting_since = Some(now);
        }
    }
}
