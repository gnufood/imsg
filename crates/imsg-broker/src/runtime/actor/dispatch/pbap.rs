//! PBAP-specific dispatch — kept separate from the MAP handlers in `super` since PBAP is a
//! distinct fault domain: `run_pbap_op` (in `super::super::serve`) treats every `Err` here as a
//! plain [`BrokerResponse::Failed`] and drops the held session for a reconnect next time, never
//! as a signal to reconnect the (unrelated) MAP session.

use anyhow::Result;
use ipc::{BrokerResponse, Reason};
use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::runtime::actor::dto::{to_card_entry_dto, to_contact_dto};

/// Pulls the main phonebook via `pbap` and upserts contact display names into the local
/// contacts cache (see `session::contacts::sync_contacts`).
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

/// Lowercases `name` and maps it to the matching [`PhonebookPath`]; `None` if unrecognized.
fn parse_path(name: &str) -> Option<PhonebookPath> {
    match name {
        "pb" => Some(PhonebookPath::Pb),
        "ich" => Some(PhonebookPath::Ich),
        "och" => Some(PhonebookPath::Och),
        "mch" => Some(PhonebookPath::Mch),
        "cch" => Some(PhonebookPath::Cch),
        "spd" => Some(PhonebookPath::Spd),
        "fav" => Some(PhonebookPath::Fav),
        _ => None,
    }
}

/// Resolves the wire `path` name to a [`PhonebookPath`] (defaulting to
/// [`PhonebookPath::Pb`][PhonebookPath::Pb] when absent), or the [`Reason`] to fail with for an
/// unrecognized name.
fn resolve_path(path: Option<&str>) -> std::result::Result<PhonebookPath, Reason> {
    path.map_or(Ok(PhonebookPath::Pb), |name| {
        parse_path(name)
            .ok_or_else(|| Reason::OperationFailed(format!("unknown phonebook path: {name}")))
    })
}

/// Lists a phonebook live and returns lean entry DTOs; no store write, pagination is
/// device-side.
///
/// # Errors
///
/// Returns an error if the PBAP listing fails.
pub(in crate::runtime::actor) async fn do_contacts_list<T: AsyncRead + AsyncWrite + Unpin>(
    pbap: &mut PbapClient<T>,
    path: Option<String>,
    limit: Option<u16>,
    offset: u16,
) -> Result<BrokerResponse> {
    let path = match resolve_path(path.as_deref()) {
        Ok(p) => p,
        Err(reason) => return Ok(BrokerResponse::Failed(reason)),
    };
    let entries = session::contacts::list(pbap, path, limit, offset).await?;
    Ok(BrokerResponse::ContactEntries(entries.iter().map(to_card_entry_dto).collect()))
}

/// Fetches one contact vCard live by handle; no store write.
///
/// # Errors
///
/// Returns an error if the PBAP pull fails (including an unresolvable handle).
pub(in crate::runtime::actor) async fn do_contacts_get<T: AsyncRead + AsyncWrite + Unpin>(
    pbap: &mut PbapClient<T>,
    path: Option<String>,
    handle: String,
) -> Result<BrokerResponse> {
    let path = match resolve_path(path.as_deref()) {
        Ok(p) => p,
        Err(reason) => return Ok(BrokerResponse::Failed(reason)),
    };
    let contact = session::contacts::get(pbap, path, &handle).await?;
    Ok(BrokerResponse::Contact(to_contact_dto(contact)))
}

/// Reverse-looks-up a contact by phone number live, then pulls its vCard; no store write.
///
/// # Errors
///
/// Returns an error if the PBAP search or pull fails.
pub(in crate::runtime::actor) async fn do_contacts_lookup<T: AsyncRead + AsyncWrite + Unpin>(
    pbap: &mut PbapClient<T>,
    path: Option<String>,
    number: String,
) -> Result<BrokerResponse> {
    let path = match resolve_path(path.as_deref()) {
        Ok(p) => p,
        Err(reason) => return Ok(BrokerResponse::Failed(reason)),
    };
    let found = session::contacts::lookup(pbap, path, &number).await?;
    Ok(BrokerResponse::ContactLookup(found.map(to_contact_dto)))
}

/// Pulls every contact vCard in a phonebook live; no store write, pagination is device-side.
///
/// # Errors
///
/// Returns an error if the PBAP pull fails.
pub(in crate::runtime::actor) async fn do_contacts_pull_all<T: AsyncRead + AsyncWrite + Unpin>(
    pbap: &mut PbapClient<T>,
    path: Option<String>,
    limit: Option<u16>,
    offset: u16,
) -> Result<BrokerResponse> {
    let path = match resolve_path(path.as_deref()) {
        Ok(p) => p,
        Err(reason) => return Ok(BrokerResponse::Failed(reason)),
    };
    let contacts = session::contacts::pull_all(pbap, path, limit, offset).await?;
    Ok(BrokerResponse::Contacts(contacts.into_iter().map(to_contact_dto).collect()))
}

#[cfg(test)]
mod tests;
