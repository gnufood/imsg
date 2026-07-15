//! `sync` — the startup gate's sync-opt-in step: ensures the local store has completed at least
//! one sync, triggering it via the broker if not.
//!
//! Called directly from `main.rs`, once, at startup — same shape as
//! `daemon::provision::ensure_running` (gate step 3), not a `#[tauri::command]`.

use store::Store;

/// Failure ensuring the store has opted into sync.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// Reading or writing the `sync_enabled` meta flag failed.
    #[error("sync_enabled meta: {0}")]
    Meta(#[from] store::Error),
    /// The broker sync request failed.
    #[error("{0}")]
    Sync(#[from] broker_client::WriteError),
}

/// Ensures `store` has completed at least one sync, triggering one via the broker at `addr` if
/// `sync_enabled` isn't already set.
///
/// Idempotent: returns immediately once `sync_enabled` is set — a startup gate step, not meant
/// to force a re-sync on every launch. Assumes the daemon is already reachable (gate step 3,
/// [`crate::daemon::provision::ensure_running`], must run first).
///
/// # Errors
///
/// Returns [`SyncError::Meta`] if the `sync_enabled` flag can't be read, or can't be written
/// after a successful sync.
/// Returns [`SyncError::Sync`] if the broker can't be reached or rejects the sync request —
/// `sync_enabled` is left unset in that case, so the next launch retries.
pub async fn ensure_synced(store: &Store, addr: &str) -> Result<(), SyncError> {
    if store.get_meta("sync_enabled").await?.as_deref() == Some("true") {
        return Ok(());
    }
    broker_client::sync(addr, None).await?;
    store.set_meta("sync_enabled", "true").await?;
    Ok(())
}

#[cfg(test)]
mod tests;
