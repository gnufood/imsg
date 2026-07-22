//! `#[tauri::command]` shims over `crate::reads`' contact queries and `crate::contacts::sync_now`.

use tauri::State;

use crate::dto::{ContactDto, ContactEntryDto};
use crate::reads;

use super::CommandError;

/// Returns lightweight cached contact identities, ordered by display name, page-limited.
///
/// # Errors
///
/// Returns [`CommandError`] if the underlying local-store read fails.
#[tauri::command]
#[specta::specta]
pub async fn list_contacts(
    store: State<'_, store::Store>,
    limit: u16,
    offset: u16,
) -> Result<Vec<ContactEntryDto>, CommandError> {
    Ok(reads::list_contacts(&store, limit, offset).await?)
}

/// Returns the full cached contact for `uid`, or `None` if uncached.
///
/// # Errors
///
/// Returns [`CommandError`] if the underlying local-store read fails.
#[tauri::command]
#[specta::specta]
pub async fn get_contact(
    store: State<'_, store::Store>,
    uid: String,
) -> Result<Option<ContactDto>, CommandError> {
    Ok(reads::get_contact(&store, &uid).await?)
}

/// Returns the full cached contact that owns phone number `address`, or `None` if unknown.
///
/// # Errors
///
/// Returns [`CommandError`] if the underlying local-store read fails.
#[tauri::command]
#[specta::specta]
pub async fn lookup_contact(
    store: State<'_, store::Store>,
    address: String,
) -> Result<Option<ContactDto>, CommandError> {
    Ok(reads::lookup_contact(&store, &address).await?)
}

/// Triggers an immediate contacts refresh via the broker, for a user-initiated "Refresh
/// contacts" action. Unlike the startup gate's best-effort pass, failure is surfaced here.
///
/// # Errors
///
/// Returns [`CommandError`] if the broker can't be reached or rejects the request.
#[tauri::command]
#[specta::specta]
pub async fn sync_contacts_now(addr: String) -> Result<usize, CommandError> {
    Ok(crate::contacts::sync_now(&addr).await?)
}

#[cfg(test)]
mod tests;
