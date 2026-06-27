//! `unsync` subcommand — opt out of local-first reads.

use std::path::PathBuf;

use anyhow::{Context, Result};
use store::Store;

/// Clears `sync_enabled` in the `meta` table, reverting `list`/`get`/`threads` to direct
/// phone access. The database file and all synced data are preserved.
///
/// # Errors
///
/// Returns an error if the store write fails.
pub(crate) async fn disable(store: &Store) -> Result<()> {
    store.set_meta("sync_enabled", "false").await?;
    Ok(())
}

/// Deletes the database file and companion WAL/SHM files without opening a connection.
///
/// The caller must ensure no open connection to the database exists before calling this;
/// open connections leave WAL inconsistencies on Unix and prevent deletion on Windows.
/// WAL/SHM removal is best-effort — errors on those files are silently ignored.
///
/// # Errors
///
/// Returns an error if the main database file cannot be removed.
pub(crate) fn purge(db_path: PathBuf) -> Result<()> {
    std::fs::remove_file(&db_path)
        .with_context(|| format!("removing database: {}", db_path.display()))?;
    // Best-effort cleanup of WAL and SHM; ignore absent files.
    let mut wal = db_path.clone().into_os_string();
    wal.push("-wal");
    let _ = std::fs::remove_file(PathBuf::from(wal));
    let mut shm = db_path.into_os_string();
    shm.push("-shm");
    let _ = std::fs::remove_file(PathBuf::from(shm));
    Ok(())
}
