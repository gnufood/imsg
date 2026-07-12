//! Local-store read path: opens the GUI's own `Store` connection (same pattern as the CLI's
//! local `list`/`get`/`threads`, bypassing the broker) and returns `dto` types.

use crate::dto::{MessageDto, ThreadDto};

/// Errors from opening the local store.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// `store.path` is unset in config and neither `HOME` nor `XDG_DATA_HOME` is set.
    #[error("no data directory available (set HOME or XDG_DATA_HOME, or configure store.path)")]
    NoDataDir,
    /// Keyring initialization or key retrieval failed.
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    /// Store open failed.
    #[error("store error: {0}")]
    Store(#[from] store::Error),
}

/// Opens the GUI's own local store connection, independent of whatever `Store` the
/// daemon/broker holds — mirrors the CLI's `open_store` (`crates/cli/src/commands/mod.rs`).
///
/// # Errors
///
/// Returns [`Error::NoDataDir`] if no data directory is configured or resolvable,
/// [`Error::Keyring`] if the Secret Service is unavailable or the stored key is corrupt, or
/// [`Error::Store`] if the database file cannot be opened or migrated.
pub async fn open_store(cfg: &config::Config) -> Result<store::Store, Error> {
    let path = cfg.store.resolve().ok_or(Error::NoDataDir)?;
    let ready = keyring::init_store()?;
    let key = keyring::get_or_create_db_key(&ready)?;
    Ok(store::Store::open(path, key).await?)
}

/// Returns the message with the given MAP handle, or `None` if absent.
///
/// # Errors
///
/// Returns [`store::Error`] if the underlying read fails.
pub async fn get_by_handle(
    db: &store::Store,
    handle: &str,
) -> Result<Option<MessageDto>, store::Error> {
    Ok(db.get_by_handle(handle).await?.as_ref().map(MessageDto::from))
}

/// Returns messages matching all supplied criteria, newest-first; see
/// [`store::Store::list_messages`] for parameter semantics.
///
/// # Errors
///
/// Returns [`store::Error`] if the underlying read fails.
pub async fn list_messages(
    db: &store::Store,
    folder: Option<&str>,
    unread_only: bool,
    from: Option<&str>,
    since_ms: Option<i64>,
    limit: u16,
    offset: u16,
) -> Result<Vec<MessageDto>, store::Error> {
    let rows = db.list_messages(folder, unread_only, from, since_ms, limit, offset).await?;
    Ok(rows.iter().map(MessageDto::from).collect())
}

/// Marks a message read in the local store. Device-side mark-read is deferred to the next
/// sync — this never opens a Bluetooth connection.
///
/// # Errors
///
/// Returns [`store::Error`] if the underlying write fails.
pub async fn mark_read(db: &store::Store, handle: &str) -> Result<(), store::Error> {
    db.update_status(handle, store::STATUS_READ).await
}

/// Returns a per-address thread summary, most-recent-first.
///
/// # Errors
///
/// Returns [`store::Error`] if the underlying read fails.
pub async fn threads(db: &store::Store) -> Result<Vec<ThreadDto>, store::Error> {
    let rows = db.threads().await?;
    Ok(rows.iter().map(ThreadDto::from).collect())
}

#[cfg(test)]
mod tests;
