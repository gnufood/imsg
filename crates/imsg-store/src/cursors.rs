//! Per-folder sync cursor operations: read and upsert progress anchors.

use rusqlite::{params, OptionalExtension};

use crate::{
    row::{FolderCursor, FolderSyncStatus},
    Error, Store,
};

impl Store {
    /// Returns the sync cursor for `folder`, or `None` if the folder has never been synced.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the query fails.
    pub async fn get_cursor(&self, folder: &str) -> Result<Option<FolderCursor>, Error> {
        let folder = folder.to_owned();
        self.conn()
            .call(move |conn| {
                conn.prepare_cached(
                    "SELECT folder, last_sync_at, highest_ts, sync_status \
                     FROM folder_cursors WHERE folder = ?1",
                )?
                .query_row([folder.as_str()], |row| {
                    Ok(FolderCursor {
                        folder: row.get(0)?,
                        last_sync_at: row.get(1)?,
                        highest_ts: row.get(2)?,
                        sync_status: row.get(3)?,
                    })
                })
                .optional()
            })
            .await
            .map_err(Error::Connection)
    }

    /// Upserts the sync cursor for `folder`; creates the row if absent, overwrites if present.
    ///
    /// `last_sync_at` and `highest_ts` are milliseconds since Unix epoch.
    /// Call with `sync_status = Complete` after a successful backfill and `Failed` on error.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the upsert fails.
    pub async fn set_cursor(
        &self,
        folder: &str,
        last_sync_at: i64,
        highest_ts: i64,
        sync_status: FolderSyncStatus,
    ) -> Result<(), Error> {
        let folder = folder.to_owned();
        self.conn()
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO folder_cursors \
                     (folder, last_sync_at, highest_ts, sync_status) \
                     VALUES (?1, ?2, ?3, ?4) \
                     ON CONFLICT(folder) DO UPDATE SET \
                         last_sync_at = excluded.last_sync_at, \
                         highest_ts   = excluded.highest_ts, \
                         sync_status  = excluded.sync_status",
                    params![folder, last_sync_at, highest_ts, sync_status],
                )?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }
}
