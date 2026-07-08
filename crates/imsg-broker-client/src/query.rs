//! Structured single-shot broker queries: persistence mode and session state.

use ipc::{BrokerRequest, BrokerResponse, SessionState};

use crate::transport::{connect_raw, recv_frame, send_frame};

/// Returns whether a reachable broker is running in persistent (daemon) mode, or `None` if no
/// broker answers at `addr`.
///
/// Distinguishes an already-running daemon from an ephemeral one-shot broker that merely
/// happens to be holding the socket — both answer a raw connect probe identically, so only
/// the `Status` response's `persistent` field tells them apart.
pub async fn query_persistent(addr: &str) -> Option<bool> {
    let Ok(mut framed) = connect_raw(addr).await else {
        tracing::debug!("broker: nothing reachable at {addr}");
        return None;
    };
    send_frame(&mut framed, &BrokerRequest::Status).await.ok()?;
    match recv_frame(&mut framed).await.ok()? {
        BrokerResponse::StatusInfo { persistent, .. } => {
            let kind = if persistent { "persistent daemon" } else { "ephemeral broker" };
            tracing::debug!("broker: found {kind} already running at {addr}");
            Some(persistent)
        }
        _ => None,
    }
}

/// Returns the current session state of a reachable broker, or `None` if nothing answers at
/// `addr` yet.
pub async fn query_state(addr: &str) -> Option<SessionState> {
    let mut framed = connect_raw(addr).await.ok()?;
    send_frame(&mut framed, &BrokerRequest::Status).await.ok()?;
    match recv_frame(&mut framed).await.ok()? {
        BrokerResponse::StatusInfo { state, .. } => Some(state),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
