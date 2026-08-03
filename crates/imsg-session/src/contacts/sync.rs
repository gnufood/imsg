//! Version-aware PBAP contact sync: skips the full pull entirely when the device reports an
//! unchanged phonebook identity/version watermark.

use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use pbap_core::PhonebookMetadata;
use store::{NewContact, PbapMeta, Store};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::util::now_ms;

/// What a [`sync_contacts`] call did.
///
/// Distinguishes the two outcomes a bare count can't: a phonebook whose watermark still matches
/// the cache (nothing fetched) from a refresh that ran and happened to write nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncReport {
    /// Device reported an unchanged `DatabaseIdentifier` + version counters, so no vCards were
    /// pulled. The cache is current, and the `contacts_synced`/`contacts_synced_at` markers are
    /// still stamped — this is a successful sync, not a skipped one.
    UpToDate,
    /// A full refresh ran; see [`Refresh`] for what it did and didn't manage to store.
    Refreshed(Refresh),
}

/// Per-entry accounting for one phonebook refresh.
///
/// `listed` is what the device offered; the rest describe what became of it. Entries are lost
/// silently on the device side (`pull_failed`) or the parse side (`no_uid`), so
/// `written < listed` is the signal that the cache is an incomplete view of the phonebook.
///
/// The counters don't sum to `listed`: PBAP's `0.vcf` owner card is listed but deliberately
/// never pulled, and it carries no counter of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refresh {
    /// Entries the device's listing reported, including the skipped owner card.
    pub listed: usize,
    /// Entries whose vCard fetch failed; logged and skipped, never fatal to the sync.
    pub pull_failed: usize,
    /// Fetched vCards carrying no `UID`. Unstorable — `UID` is the cache's primary key.
    pub no_uid: usize,
    /// Contacts handed to the store.
    pub written: usize,
    /// `true` when a *previously established* `DatabaseIdentifier` changed, discarding the whole
    /// cache before this refresh — every cached UID was invalidated rather than updated in place.
    /// A first-ever sync leaves this `false`: it has no prior identity to invalidate.
    pub wiped: bool,
}

// hex-encodes a 16-byte PBAP identifier/counter for storage in the text-only meta table
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

// skips PBAP's 0.vcf owner card, and any entry whose pull fails or whose vCard carries no UID
// (the store's primary key) — a per-entry failure is logged, counted, and doesn't abort the
// sync. wiped is passed through to the report rather than decided here; the cache-wipe
// decision belongs to sync_contacts, which owns the watermark comparison
async fn refresh_contacts<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    store: &Store,
    path: PhonebookPath,
    wiped: bool,
) -> anyhow::Result<Refresh> {
    let entries = client.list(path, None, 0).await?;
    let listed = entries.len();
    let mut pull_failed = 0usize;
    let mut fetched = Vec::with_capacity(listed);
    for entry in &entries {
        if entry.handle() == "0.vcf" {
            continue;
        }
        match client.pull(path, entry.handle()).await {
            Ok(contact) => fetched.push(contact),
            Err(e) => {
                pull_failed = pull_failed.saturating_add(1);
                tracing::warn!("pbap pull {} failed, skipping: {e}", entry.handle());
            }
        }
    }
    let pulled = fetched.len();
    let new_contacts: Vec<NewContact> = fetched
        .into_iter()
        .filter_map(|c| {
            let phones = c.phones().to_vec();
            Some(NewContact { uid: c.uid?, display_name: c.display_name, phones })
        })
        .collect();
    let written = new_contacts.len();
    store.upsert_contacts(new_contacts).await?;
    Ok(Refresh { listed, pull_failed, no_uid: pulled.saturating_sub(written), written, wiped })
}

/// Syncs the store's contact cache with the device's phonebook at `path`, skipping the refresh
/// when the device reports an unchanged `DatabaseIdentifier` + version counters.
///
/// Fetches a metadata-only listing first (no vCard bodies). If the identity/version watermark
/// matches the cached one, returns [`SyncReport::UpToDate`] with no further requests. A changed
/// `DatabaseIdentifier` wipes the cache before refreshing (previously cached UIDs may no longer
/// be valid); a version-counter-only change refreshes without wiping. A `DatabaseIdentifier` the
/// device never reports (`None`) can't establish cache validity, so every call refreshes.
///
/// On any success path (including the no-op), sets the `contacts_synced` meta flag — the
/// contacts-domain counterpart to `sync_enabled`, read by callers deciding whether the local
/// contacts cache is trustworthy enough to read directly — and stamps
/// [`Store::set_contacts_synced_at`] for freshness display.
///
/// # Errors
///
/// Returns an error if the metadata fetch, phonebook refresh, or any store write fails. Failures
/// pulling an individual vCard are reported in [`Refresh`], not returned here.
pub async fn sync_contacts<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    store: &Store,
    path: PhonebookPath,
) -> anyhow::Result<SyncReport> {
    let remote = to_pbap_meta(&client.phonebook_metadata(path).await?);
    let cached = store.pbap_meta().await?;

    if remote.database_id.is_some() && remote == cached {
        store.set_meta("contacts_synced", "true").await?;
        store.set_contacts_synced_at(now_ms()).await?;
        return Ok(SyncReport::UpToDate);
    }
    let cleared = remote.database_id.is_some() && remote.database_id != cached.database_id;
    if cleared {
        store.clear_contacts().await?;
    }
    // A first-ever sync also "clears" (an empty cache), but nothing was invalidated by it —
    // only report a wipe when a previously established identity was replaced.
    let wiped = cleared && cached.database_id.is_some();

    let refresh = refresh_contacts(client, store, path, wiped).await?;
    store.set_pbap_meta(&remote).await?;
    store.set_meta("contacts_synced", "true").await?;
    store.set_contacts_synced_at(now_ms()).await?;
    Ok(SyncReport::Refreshed(refresh))
}

#[cfg(test)]
mod tests;
