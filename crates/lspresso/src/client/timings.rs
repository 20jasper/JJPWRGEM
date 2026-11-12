use std::time::{Duration, Instant};

/// Where the time between sending a message and holding the server's next
/// message in full went. None of it is the client's own serialization or
/// parsing, which is what wrapping a call in [`Instant`] would add.
///
/// The counters only grow; measure a call by taking [`Timings::since`] across
/// it. `total` is the sum of the other three.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Timings {
    pub total: Duration,
    /// Writing the message to the server's stdin. A server reads while this
    /// happens, so for large messages some of its work overlaps
    pub request_io: Duration,
    /// From the message being fully written to the first byte of the next
    /// message from the server
    pub server: Duration,
    /// From that first byte to the last
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
