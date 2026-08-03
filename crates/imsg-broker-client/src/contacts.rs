//! `SyncContacts` — the one PBAP write op, mirroring `write.rs`'s `send`/`delete`/`sync`.
//!
//! Live PBAP reads (`list`/`get`/`lookup`/`pull_all`) don't get typed helpers here — they
//! follow `list.rs`/`get.rs`/`threads.rs`'s existing pattern instead: call
//! `commands::broker::call` directly and match the response inline at the CLI call site.

use ipc::{BrokerRequest, SyncReportDto};

use crate::response::{contacts_synced_result, CallError};
use crate::transport::send_request;

/// Failure sending a broker contacts request.
#[derive(Debug, thiserror::Error)]
pub enum ContactsError {
    /// The broker couldn't be reached, or the request/response frame itself was malformed.
    #[error("{0}")]
    Connect(#[from] anyhow::Error),
    /// The broker was reachable but rejected the request or answered unexpectedly.
    #[error(transparent)]
    Call(#[from] CallError),
}

/// Pulls the main phonebook and upserts contact display names into the broker's local cache,
/// returning what the sync did — see [`SyncReportDto`].
///
/// # Errors
///
/// Returns [`ContactsError::Connect`] if the broker can't be reached, or
/// [`ContactsError::Call`] if it rejects the request.
pub async fn sync_contacts(addr: &str) -> Result<SyncReportDto, ContactsError> {
    let resp = send_request(addr, BrokerRequest::SyncContacts).await?;
    Ok(contacts_synced_result(resp)?)
}

#[cfg(test)]
mod tests;
