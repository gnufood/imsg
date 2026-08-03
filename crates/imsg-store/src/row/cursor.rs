use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use strum_macros::{Display, EnumIs, EnumString};

/// Completion state stored in `folder_cursors.sync_status`.
///
/// `never` is the initial value before any sync attempt on a given folder.
/// `complete` means the last sync finished without error and `highest_ts` is reliable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIs)]
#[strum(serialize_all = "snake_case")]
pub enum FolderSyncStatus {
    /// No sync has been attempted for this folder yet.
    Never,
    /// A sync is currently in progress.
    Syncing,
    /// Last sync completed successfully.
    Complete,
    /// Last sync ended with an error; `highest_ts` reflects the last successful boundary.
    Failed,
}

impl ToSql for FolderSyncStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for FolderSyncStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = String::column_result(value)?;
        s.parse().map_err(|_| FromSqlError::InvalidType)
    }
}

/// Per-folder sync cursor stored in `folder_cursors`.
///
/// Replaces the single global `last_sync_at` anchor in the `meta` table. Each folder
/// tracks its own progress so a partial sync on one folder never corrupts another.
#[derive(Debug, Clone)]
pub struct FolderCursor {
    /// MAP folder leaf (e.g. `"inbox"`, `"sent"`).
    pub folder: String,
    /// Milliseconds since Unix epoch of the last completed sync run for this folder.
    pub last_sync_at: i64,
    /// Highest `timestamp_ms` seen during the last sync; used as the next pull boundary.
    pub highest_ts: i64,
    /// Whether the last sync completed, is in progress, or failed.
    pub sync_status: FolderSyncStatus,
}
