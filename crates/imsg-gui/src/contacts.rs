//! `contacts` — the startup gate's best-effort contacts-sync step.
//!
//! Unlike `sync::ensure_synced` (MAP), this runs every gate pass rather than once-ever: PBAP has
//! no push/poll channel, so a manual trigger is the only way the contacts cache ever refreshes,
//! and `session::contacts::sync_contacts`'s own `DatabaseIdentifier`/version-counter check keeps
//! a no-change call cheap (metadata round-trip only). Failure is logged and swallowed rather than
//! propagated — contacts are supplementary display data, not required for the app to be usable,
//! so this can never block the gate the way a MAP sync failure does.

/// Best-effort contacts refresh via the broker at `addr`. Never returns an error; any failure is
/// logged and the next gate pass (next launch, or a `gate_proceed` poke) retries automatically.
pub async fn ensure_synced_best_effort(addr: &str) {
    if let Err(e) = broker_client::sync_contacts(addr).await {
        tracing::warn!("contacts sync failed (non-fatal): {e:#}");
    }
}

/// Triggers an immediate contacts refresh via the broker at `addr`, reporting what it did.
///
/// Surfaces any failure to the caller — unlike [`ensure_synced_best_effort`], meant for a
/// user-initiated "Refresh contacts" action rather than the silent, retried-next-launch gate step.
///
/// # Errors
///
/// Returns [`broker_client::ContactsError`] if the broker can't be reached or rejects the request.
pub async fn sync_now(addr: &str) -> Result<ipc::SyncReportDto, broker_client::ContactsError> {
    broker_client::sync_contacts(addr).await
}

#[cfg(test)]
mod tests;
