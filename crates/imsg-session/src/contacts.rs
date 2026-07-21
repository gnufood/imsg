//! Store-integrated PBAP contact sync.

use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};

/// Pulls the full phonebook at `path` and upserts each contact's phone numbers into the store.
///
/// One row per number, keyed exactly as the device's vCard reports it — no normalization,
/// matching `messages.address`'s own raw storage so `Store::threads`'s join resolves.
/// Contacts with no phone numbers contribute no rows. This is additions/updates only: a
/// number no longer present on the device is left untouched in the store, since PBAP has no
/// change-set or removal signal to act on.
///
/// Returns the number of address rows upserted.
///
/// # Errors
///
/// Returns an error if the PBAP pull or any store write fails.
pub async fn sync_contacts<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    store: &Store,
    path: PhonebookPath,
) -> anyhow::Result<usize> {
    let contacts = client.pull_all(path, None, 0).await?;
    let entries: Vec<(String, Option<String>)> = contacts
        .iter()
        .flat_map(|c| c.phones().iter().map(move |p| (p.clone(), c.display_name.clone())))
        .collect();
    store.upsert_contacts(entries).await.map_err(Into::into)
}

#[cfg(test)]
mod tests;
