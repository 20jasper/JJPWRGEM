use std::time::{Duration, Instant};

/// measures granular time spent receiving, processing, and responding to requests. Real
/// clients vary in speed so split out the language server's work from IO
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
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

    pub(super) fn record(
        &mut self,
        waiting_since: Instant,
        queued_at: Option<Instant>,
        written_at: Option<Instant>,
        first_byte_at: Instant,
        parsed_at: Instant,
    ) {
        let parsed_at = parsed_at.max(waiting_since);
        let first_byte_at = first_byte_at.clamp(waiting_since, parsed_at);
        let written_at = written_at
            .or(queued_at)
            .map_or(waiting_since, |at| at.clamp(waiting_since, first_byte_at));
        self.total += parsed_at - waiting_since;
        self.request += written_at - waiting_since;
        self.server += first_byte_at - written_at;
        self.response += parsed_at - first_byte_at;
    }
}
