//! Version-aware PBAP contact sync: skips the full pull entirely when the device reports an
//! unchanged phonebook identity/version watermark.

use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use pbap_core::PhonebookMetadata;
use store::{NewContact, PbapMeta, Store};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::util::now_ms;

/// Hex-encodes a 16-byte PBAP identifier/counter for storage in the text-only `meta` table.
fn hex16(bytes: [u8; 16]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(32);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn to_pbap_meta(m: &PhonebookMetadata) -> PbapMeta {
    PbapMeta {
        database_id: m.database_id.map(hex16),
        primary_version: m.primary_version.map(hex16),
        secondary_version: m.secondary_version.map(hex16),
    }
}

/// Pulls the full phonebook at `path` and upserts every contact into the store.
///
/// Skips PBAP's `0.vcf` owner card, and any entry whose pull fails or whose vCard carries no
/// `UID` (the store's primary key) — a per-entry failure is logged and does not abort the sync.
async fn refresh_contacts<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    store: &Store,
    path: PhonebookPath,
) -> anyhow::Result<usize> {
    let entries = client.list(path, None, 0).await?;
    let mut fetched = Vec::with_capacity(entries.len());
    for entry in &entries {
        if entry.handle() == "0.vcf" {
            continue;
        }
        match client.pull(path, entry.handle()).await {
            Ok(contact) => fetched.push(contact),
            Err(e) => tracing::warn!("pbap pull {} failed, skipping: {e}", entry.handle()),
        }
    }
    let new_contacts: Vec<NewContact> = fetched
        .into_iter()
        .filter_map(|c| {
            let phones = c.phones().to_vec();
            Some(NewContact { uid: c.uid?, display_name: c.display_name, phones })
        })
        .collect();
    Ok(store.upsert_contacts(new_contacts).await?)
}

/// Syncs the store's contact cache with the device's phonebook at `path`, skipping the refresh
/// when the device reports an unchanged `DatabaseIdentifier` + version counters.
///
/// Fetches a metadata-only listing first (no vCard bodies). If the identity/version watermark
/// matches the cached one, returns `Ok(0)` with no further requests. A changed
/// `DatabaseIdentifier` wipes the cache before refreshing (previously cached UIDs may no longer
/// be valid); a version-counter-only change refreshes without wiping. A `DatabaseIdentifier` the
/// device never reports (`None`) can't establish cache validity, so every call refreshes.
///
/// Returns the number of contacts upserted. On any success path (including the no-op), sets the
/// `contacts_synced` meta flag — the contacts-domain counterpart to `sync_enabled`, read by
/// callers deciding whether the local contacts cache is trustworthy enough to read directly —
/// and stamps [`Store::set_contacts_synced_at`] for freshness display.
///
/// # Errors
///
/// Returns an error if the metadata fetch, phonebook refresh, or any store write fails.
pub async fn sync_contacts<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    store: &Store,
    path: PhonebookPath,
) -> anyhow::Result<usize> {
    let remote = to_pbap_meta(&client.phonebook_metadata(path).await?);
    let cached = store.pbap_meta().await?;

    if remote.database_id.is_some() && remote == cached {
        store.set_meta("contacts_synced", "true").await?;
        store.set_contacts_synced_at(now_ms()).await?;
        return Ok(0);
    }
    if remote.database_id.is_some() && remote.database_id != cached.database_id {
        store.clear_contacts().await?;
    }

    let count = refresh_contacts(client, store, path).await?;
    store.set_pbap_meta(&remote).await?;
    store.set_meta("contacts_synced", "true").await?;
    store.set_contacts_synced_at(now_ms()).await?;
    Ok(count)
}

#[cfg(test)]
mod tests;
