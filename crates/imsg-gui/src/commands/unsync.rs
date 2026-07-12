//! `#[tauri::command]` shim over `crate::unsync::disable`.
//!
//! `unsync --purge` (delete the DB file) has no command here yet: it requires the caller to
//! guarantee no open connection to the database exists first (see `crate::unsync::purge`'s
//! doc), which the CLI can do trivially (one-shot process, no long-lived connection) but the
//! GUI can't yet — its `Store` is meant to be long-lived Tauri-managed `State`, and nothing in
//! `main.rs`'s (unbuilt) state lifecycle currently supports safely dropping and reopening it.
//! Revisit once that's designed.

use tauri::State;

use super::CommandError;

/// Clears `sync_enabled`, reverting reads to the live/broker path. Preserves the store's data.
///
/// # Errors
///
/// Returns [`CommandError`] if the store write fails.
#[tauri::command]
#[specta::specta]
pub async fn unsync_disable(store: State<'_, store::Store>) -> Result<(), CommandError> {
    Ok(crate::unsync::disable(&store).await?)
}

#[cfg(test)]
mod tests;
