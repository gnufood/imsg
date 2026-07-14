//! Requests that must reach the live device (`Send`, `Delete`, `Sync`) — the daemon holds the
//! sole `MapClient`, so these always go over IPC regardless of any client-side read cache.

use ipc::BrokerRequest;

use crate::response::{text_result, CallError};
use crate::transport::send_request;

/// Failure sending a broker write request.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// The broker couldn't be reached, or the request/response frame itself was malformed.
    #[error("{0}")]
    Connect(#[from] anyhow::Error),
    /// The broker was reachable but rejected the request or answered unexpectedly.
    #[error(transparent)]
    Call(#[from] CallError),
}

/// Records and pushes an outgoing SMS; the broker enqueues it in its own store's outbox and
/// tracks delivery.
///
/// # Errors
///
/// Returns [`WriteError::Connect`] if the broker can't be reached, or [`WriteError::Call`] if
/// it rejects the request.
pub async fn send(addr: &str, number: String, message: String) -> Result<String, WriteError> {
    let resp = send_request(addr, BrokerRequest::Send { number, message }).await?;
    Ok(text_result(resp)?)
}

/// Sets the deleted flag on the device and removes the message from the broker's store.
///
/// # Errors
///
/// Returns [`WriteError::Connect`] if the broker can't be reached, or [`WriteError::Call`] if
/// it rejects the request.
pub async fn delete(addr: &str, handle: String, folder: String) -> Result<String, WriteError> {
    let resp = send_request(addr, BrokerRequest::Delete { handle, folder }).await?;
    Ok(text_result(resp)?)
}

/// Backfills MAP folders since their per-folder cursor anchors. `folder` limits the backfill to
/// one MAP path; `None` syncs all four standard folders.
///
/// # Errors
///
/// Returns [`WriteError::Connect`] if the broker can't be reached, or [`WriteError::Call`] if
/// it rejects the request.
pub async fn sync(addr: &str, folder: Option<String>) -> Result<String, WriteError> {
    let resp = send_request(addr, BrokerRequest::Sync { folder }).await?;
    Ok(text_result(resp)?)
}

#[cfg(test)]
mod tests;
