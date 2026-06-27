//! Connection-state and failure-reason wire types.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The broker's MAP session lifecycle state, as reported to the CLI over [`crate::BrokerResponse`].
///
/// Describes the MAP/OBEX session only — never MNS/notification health. Progression is
/// `Disconnected → Connecting → Active`, with `Active → Reconnecting → Active` on a recoverable
/// drop and a terminal `Failed` when the connect budget is exhausted or a permanent error occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// No connection attempted yet.
    Disconnected,
    /// Establishing the RFCOMM/OBEX/MAP session (first attempt or a reconnect attempt).
    Connecting,
    /// MAP session live; operations may run.
    Active,
    /// A live session dropped; backoff is running before the next attempt.
    Reconnecting,
    /// Terminal: the connect budget was exhausted or a permanent error occurred. The broker exits.
    Failed,
}

impl SessionState {
    /// Returns `true` only in [`SessionState::Active`] — i.e. when MAP operations can run now.
    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for SessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Disconnected => "disconnected",
            Self::Connecting => "connecting",
            Self::Active => "connected",
            Self::Reconnecting => "reconnecting",
            Self::Failed => "failed",
        };
        f.write_str(s)
    }
}

/// An action-oriented failure reason returned in [`crate::BrokerResponse::Failed`].
///
/// Each variant maps to a distinct user action; establishment *stage* (RFCOMM vs OBEX vs
/// notification registration) is deliberately not encoded — it is diagnostic log detail, not a
/// reason, because it does not change what the user should do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", content = "detail", rename_all = "snake_case")]
pub enum Reason {
    /// The session was not `Active` before the request's deadline elapsed. Action: retry.
    NotReady,
    /// The device could not be reached (link timeout/reset). Action: bring the phone close, check Bluetooth.
    DeviceUnreachable,
    /// The device refused the connection (auth/pairing/wrong channel). Action: re-pair or fix config.
    ConnectionRefused,
    /// A MAP operation was rejected by the device; the inner string is the device-reported detail.
    OperationFailed(String),
    /// An unexpected protocol/parse error; the inner string is diagnostic detail to report.
    Internal(String),
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReady => f.write_str("broker not ready — still connecting; try again"),
            Self::DeviceUnreachable => {
                f.write_str("device unreachable — bring the phone close and check Bluetooth")
            }
            Self::ConnectionRefused => {
                f.write_str("device refused the connection — re-pair or check the MAP channel")
            }
            Self::OperationFailed(d) => write!(f, "operation failed: {d}"),
            Self::Internal(d) => write!(f, "internal error: {d}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_state_connected_only_when_active() {
        assert!(SessionState::Active.is_connected());
        for s in [
            SessionState::Disconnected,
            SessionState::Connecting,
            SessionState::Reconnecting,
            SessionState::Failed,
        ] {
            assert!(!s.is_connected());
        }
    }

    #[test]
    fn reason_roundtrips_through_json() -> Result<(), serde_json::Error> {
        for r in [
            Reason::NotReady,
            Reason::DeviceUnreachable,
            Reason::ConnectionRefused,
            Reason::OperationFailed("rejected handle 7".into()),
            Reason::Internal("bad xml".into()),
        ] {
            let json = serde_json::to_string(&r)?;
            let back: Reason = serde_json::from_str(&json)?;
            assert_eq!(r, back);
        }
        Ok(())
    }

    #[test]
    fn session_state_roundtrips_through_json() -> Result<(), serde_json::Error> {
        for s in [SessionState::Connecting, SessionState::Active, SessionState::Failed] {
            let json = serde_json::to_string(&s)?;
            let back: SessionState = serde_json::from_str(&json)?;
            assert_eq!(s, back);
        }
        Ok(())
    }
}
