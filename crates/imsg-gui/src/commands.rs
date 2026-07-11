//! `#[tauri::command]` shims over `reads.rs`.
//!
//! This is the invoke-handler boundary a `tauri_specta::Builder` wraps (see `commands/tests.rs`
//! for a `collect_commands!`/`MockRuntime` smoke test). Bodies stay thin: extract the managed
//! `Store`, delegate to `reads`, map the error.

use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::dto::{MessageDto, ThreadDto};
use crate::reads;

/// Failure surfaced to the frontend for any local-store read error.
///
/// Wraps `store::Error`'s `Display` text; those messages already avoid leaking secrets (no raw
/// key material, no `SQLite` internals beyond a driver-level description), so reusing them here
/// needs no extra redaction.
#[derive(Debug, Serialize, Type, thiserror::Error)]
#[error("{message}")]
pub struct CommandError {
    message: String,
}

impl From<store::Error> for CommandError {
    fn from(err: store::Error) -> Self {
        Self { message: err.to_string() }
    }
}

/// Returns the message with the given MAP handle, or `None` if absent.
///
/// # Errors
///
/// Returns [`CommandError`] if the underlying local-store read fails.
#[tauri::command]
#[specta::specta]
pub async fn get_by_handle(
    store: State<'_, store::Store>,
    handle: String,
) -> Result<Option<MessageDto>, CommandError> {
    Ok(reads::get_by_handle(&store, &handle).await?)
}

/// Returns messages matching all supplied criteria, newest-first; see
/// [`store::Store::list_messages`] for parameter semantics.
///
/// # Errors
///
/// Returns [`CommandError`] if the underlying local-store read fails.
#[tauri::command]
#[specta::specta]
pub async fn list_messages(
    store: State<'_, store::Store>,
    folder: Option<String>,
    unread_only: bool,
    from: Option<String>,
    since_ms: Option<i64>,
    limit: u16,
    offset: u16,
) -> Result<Vec<MessageDto>, CommandError> {
    Ok(reads::list_messages(
        &store,
        folder.as_deref(),
        unread_only,
        from.as_deref(),
        since_ms,
        limit,
        offset,
    )
    .await?)
}

/// Returns a per-address thread summary, most-recent-first.
///
/// # Errors
///
/// Returns [`CommandError`] if the underlying local-store read fails.
#[tauri::command]
#[specta::specta]
pub async fn threads(store: State<'_, store::Store>) -> Result<Vec<ThreadDto>, CommandError> {
    Ok(reads::threads(&store).await?)
}

#[cfg(test)]
mod tests;
