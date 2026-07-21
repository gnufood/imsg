//! Live PBAP operations — `list`/`get`/`pull_all`/`lookup` read straight from the device, no
//! store writes. Mirrors `session::live`'s relationship to `map_core`: both the CLI hub path
//! and the broker's dispatch call these instead of `PbapClient` methods directly.

use pbap_core::client::PbapClient;
use pbap_core::phonebook::{PhonebookPath, SearchAttribute};
use pbap_core::{CardEntry, Contact};
use tokio::io::{AsyncRead, AsyncWrite};

/// Lists contact identities in `path`, windowed by `limit`/`offset`. No vCard bodies fetched.
///
/// # Errors
///
/// Returns an error if the PBAP listing fails.
pub async fn list<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    limit: Option<u16>,
    offset: u16,
) -> anyhow::Result<Vec<CardEntry>> {
    Ok(client.list(path, limit, offset).await?)
}

/// Fetches one contact's full vCard by its current PBAP handle.
///
/// `handle` is volatile — re-resolved on every [`list`] call — so callers must not cache it
/// across syncs; see `formats::vcard::Contact::uid` for the durable alternative.
///
/// # Errors
///
/// Returns an error if the PBAP pull fails.
pub async fn get<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    handle: &str,
) -> anyhow::Result<Contact> {
    Ok(client.pull(path, handle).await?)
}

/// Fetches every contact in `path`, windowed by `limit`/`offset`.
///
/// # Errors
///
/// Returns an error if the PBAP pull fails.
pub async fn pull_all<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    limit: Option<u16>,
    offset: u16,
) -> anyhow::Result<Vec<Contact>> {
    Ok(client.pull_all(path, limit, offset).await?)
}

/// Reverse-looks-up a contact by phone number via device-side `SearchAttribute::Number`, then
/// pulls its full vCard. Returns `None` if the device reports no match.
///
/// Matching is whatever the device's own search implements — unlike the removed
/// `PbapClient::find_by_number`, no client-side E.164 normalization is applied here.
///
/// # Errors
///
/// Returns an error if the PBAP search or pull fails.
pub async fn lookup<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    number: &str,
) -> anyhow::Result<Option<Contact>> {
    let entries = client.search(path, SearchAttribute::Number, number, None, 0).await?;
    let Some(entry) = entries.iter().find(|e| e.handle() != "0.vcf") else {
        return Ok(None);
    };
    Ok(Some(client.pull(path, entry.handle()).await?))
}

#[cfg(test)]
mod tests;
