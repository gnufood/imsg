//! Connection-retry primitives: transient/permanent classification and the backoff schedule.
//!
//! These are the policy pieces the broker actor *drives*; the reconnect loop that consumes them
//! lives in `imsg-broker`, not here. Keeping classification in session is mandatory — it is the
//! only layer that knows what `MapError`/`ObexError`/`io::ErrorKind` mean.

use std::io;
use std::time::Duration;

use map_core::{MapError, ObexError};
use tokio_retry::strategy::ExponentialBackoff;

use crate::{SessionError, TransportError};

/// Whether a failed connection attempt should be retried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    /// Recoverable (link timeout/reset, transient protocol hiccup) — retry with backoff.
    Transient,
    /// Non-recoverable (device refused, auth/pairing, bad input) — fail fast.
    Permanent,
}

/// Classifies a session-establishment failure as transient (retry) or permanent (fail fast).
///
/// Inspects the inner `io::ErrorKind` and OBEX/server opcodes rather than the top-level variant,
/// so "device asleep / out of range" (transient) is distinguished from "wrong channel / pairing
/// rejected" (permanent). Anything not explicitly recognised defaults to [`Disposition::Transient`]:
/// the retry budget bounds the cost of a wrong guess, whereas a wrong "permanent" verdict abandons
/// a recoverable link.
#[must_use]
pub fn classify(e: &SessionError) -> Disposition {
    match e {
        SessionError::Transport(t) => classify_transport(t),
        SessionError::Map(m) => classify_map(m),
        SessionError::Pbap(_) => Disposition::Transient,
    }
}

fn classify_map(e: &MapError) -> Disposition {
    match e {
        MapError::Transport(t) => classify_transport(t),
        // OBEX CONNECT refused, server refusal, or rejected input: retrying will not help.
        MapError::Obex(ObexError::ConnectRejected(_))
        | MapError::ServerError(_)
        | MapError::InvalidInput(_) => Disposition::Permanent,
        _ => Disposition::Transient,
    }
}

fn classify_transport(e: &TransportError) -> Disposition {
    match e {
        TransportError::Io(io) => classify_io(io.kind()),
        _ => Disposition::Transient,
    }
}

const fn classify_io(kind: io::ErrorKind) -> Disposition {
    match kind {
        // No service on the channel, auth denied, or a malformed address — all permanent.
        io::ErrorKind::ConnectionRefused
        | io::ErrorKind::PermissionDenied
        | io::ErrorKind::InvalidInput => Disposition::Permanent,
        _ => Disposition::Transient,
    }
}

/// Builds the inter-attempt backoff schedule: delays doubling from `initial`, capped at `max`,
/// yielding exactly `max_attempts - 1` values (the gaps between `max_attempts` attempts).
///
/// Empty when `max_attempts <= 1`. No jitter — the kernel-atomic bind election guarantees one
/// broker per device, so there is no thundering herd to spread. The returned iterator is itself
/// `#[must_use]`.
pub fn backoff(
    initial: Duration,
    max: Duration,
    max_attempts: u32,
) -> impl Iterator<Item = Duration> + Clone {
    let initial_ms = u64::try_from(initial.as_millis()).unwrap_or(u64::MAX);
    // base=2 gives a doubling ratio; factor scales the first delay to `initial` (2 * factor).
    let factor = (initial_ms / 2).max(1);
    let gaps = usize::try_from(max_attempts.saturating_sub(1)).unwrap_or(usize::MAX);
    ExponentialBackoff::from_millis(2).factor(factor).max_delay(max).take(gaps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn io(kind: io::ErrorKind) -> SessionError {
        SessionError::Transport(TransportError::Io(io::Error::new(kind, "x")))
    }

    #[test]
    fn timeouts_and_resets_are_transient() {
        for k in
            [io::ErrorKind::TimedOut, io::ErrorKind::ConnectionReset, io::ErrorKind::BrokenPipe]
        {
            assert_eq!(classify(&io(k)), Disposition::Transient);
        }
    }

    #[test]
    fn refusal_and_auth_are_permanent() {
        for k in [io::ErrorKind::ConnectionRefused, io::ErrorKind::PermissionDenied] {
            assert_eq!(classify(&io(k)), Disposition::Permanent);
        }
    }

    #[test]
    fn obex_connect_rejected_is_permanent() {
        let e = SessionError::Map(MapError::Obex(ObexError::ConnectRejected(0xC3)));
        assert_eq!(classify(&e), Disposition::Permanent);
    }

    #[test]
    fn server_error_is_permanent_but_eof_is_transient() {
        assert_eq!(
            classify(&SessionError::Map(MapError::ServerError(0xC0))),
            Disposition::Permanent
        );
        assert_eq!(classify(&SessionError::Map(MapError::UnexpectedEof)), Disposition::Transient);
    }

    #[test]
    fn backoff_doubles_from_initial_capped_and_bounded() {
        let delays: Vec<_> =
            backoff(Duration::from_millis(500), Duration::from_secs(2), 5).collect();
        // 500ms → 1s → 2s (cap) → 2s (cap); exactly max_attempts - 1 = 4 gaps.
        assert_eq!(
            delays.as_slice(),
            [
                Duration::from_millis(500),
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(2),
            ]
        );
    }

    #[test]
    fn backoff_is_empty_for_single_attempt() {
        assert_eq!(backoff(Duration::from_millis(10), Duration::from_secs(1), 1).count(), 0);
    }
}
