use std::time::{Duration, Instant};

/// measures granular time spent receiving, processing, and responding to requests. Real
/// clients vary in speed so split out the language server's work from IO
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Timings {
    pub total: Duration,
    /// client time spent sending and serializing the message until the server has all bytes
    pub request_io: Duration,
    /// server time spent processing request until sending response
    pub server: Duration,
    /// client time spent receiving and deserializing response from the language server
    pub response_io: Duration,
}

impl Timings {
    #[must_use]
    pub const fn since(self, earlier: Self) -> Self {
        Self {
            total: self.total.saturating_sub(earlier.total),
            request_io: self.request_io.saturating_sub(earlier.request_io),
            server: self.server.saturating_sub(earlier.server),
            response_io: self.response_io.saturating_sub(earlier.response_io),
        }
    }

    pub(super) fn record(
        &mut self,
        waiting_since: Instant,
        written_at: Option<Instant>,
        first_byte_at: Instant,
        arrived_at: Instant,
    ) {
        let arrived_at = arrived_at.max(waiting_since);
        let first_byte_at = first_byte_at.clamp(waiting_since, arrived_at);
        let written_at =
            written_at.map_or(waiting_since, |at| at.clamp(waiting_since, first_byte_at));
        self.total += arrived_at - waiting_since;
        self.request_io += written_at - waiting_since;
        self.server += first_byte_at - written_at;
        self.response_io += arrived_at - first_byte_at;
    }
}
