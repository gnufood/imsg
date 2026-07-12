//! Opt out of local-first reads — mirrors the CLI's `unsync` (`crates/cli/src/commands/
//! unsync.rs`) exactly; no broker involved, just direct `imsg-store` calls.

use std::path::PathBuf;

/// Clears `sync_enabled` in the `meta` table, reverting reads to the live/broker path.
/// The database file and all synced data are preserved.
///
/// # Errors
///
/// Returns [`store::Error`] if the store write fails.
pub async fn disable(store: &store::Store) -> Result<(), store::Error> {
    store.set_meta("sync_enabled", "false").await?;
    Ok(())
}

/// Deletes the database file and companion WAL/SHM files without opening a connection.
///
/// The caller must ensure no open connection to the database exists before calling this —
/// open connections leave WAL inconsistencies on Unix and prevent deletion on Windows. WAL/SHM
/// removal is best-effort; errors on those files are silently ignored.
///
/// # Errors
///
/// Returns an error if the main database file cannot be removed.
pub fn purge(db_path: PathBuf) -> std::io::Result<()> {
    std::fs::remove_file(&db_path)?;
    let mut wal = db_path.clone().into_os_string();
    wal.push("-wal");
    let _ = std::fs::remove_file(PathBuf::from(wal));
    let mut shm = db_path.into_os_string();
    shm.push("-shm");
    let _ = std::fs::remove_file(PathBuf::from(shm));
    Ok(())
}

#[cfg(test)]
mod tests;
