//! PBAP-specific dispatch — kept separate from the MAP handlers in `super` since it has no
//! session-lifecycle relationship to the MAP client.

use anyhow::Result;
use ipc::BrokerResponse;
use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};

/// Pulls the main phonebook via `pbap` and upserts contact display names into the local
/// contacts cache (see `session::contacts::sync_contacts`).
///
/// Unlike the MAP handlers in `super`, `pbap` is a fresh, short-lived connection opened and torn
/// down per request — there is no persistent PBAP session whose health this needs to protect,
/// so callers should treat any `Err` here as a plain [`BrokerResponse::Failed`], never as a
/// signal to reconnect the (unrelated) MAP session.
///
/// # Errors
///
/// Returns an error if the PBAP pull or any store write fails.
pub(in crate::runtime::actor) async fn do_sync_contacts<T: AsyncRead + AsyncWrite + Unpin>(
    pbap: &mut PbapClient<T>,
    store: &Store,
) -> Result<BrokerResponse> {
    let count = session::contacts::sync_contacts(pbap, store, PhonebookPath::Pb).await?;
    Ok(BrokerResponse::ContactsSynced { count })
}

#[cfg(test)]
mod tests;
