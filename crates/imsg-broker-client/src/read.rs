//! Live device reads that return typed rows rather than text.
//!
//! Unlike `list`/`get`/`threads` — which predate this module and still match the response
//! inline at their CLI call sites — these are shared helpers, so the CLI and the GUI read the
//! device through the same code path instead of each interpreting the protocol themselves.

use ipc::{BrokerRequest, FolderDto};

use crate::response::{folders_result, CallError};
use crate::transport::send_request;

/// Failure sending a broker live-read request.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    /// The broker couldn't be reached, or the request/response frame itself was malformed.
    #[error("{0}")]
    Connect(#[from] anyhow::Error),
    /// The broker was reachable but rejected the request or answered unexpectedly.
    #[error(transparent)]
    Call(#[from] CallError),
}

/// Lists the device's MAP message folders through the broker, in device-reported document order.
///
/// Goes through the broker rather than opening a MAP connection directly, so it shares the one
/// RFCOMM channel the broker or daemon already holds instead of competing with it.
///
/// # Errors
///
/// Returns [`ReadError::Connect`] if the broker can't be reached, or [`ReadError::Call`] if it
/// rejects the request.
pub async fn folders(addr: &str) -> Result<Vec<FolderDto>, ReadError> {
    let resp = send_request(addr, BrokerRequest::Folders).await?;
    Ok(folders_result(resp)?)
}

#[cfg(test)]
mod tests;
