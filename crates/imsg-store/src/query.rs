use rusqlite::params;

use crate::{row::MessageRow, Error, NewMessage, Store};

impl Store {
    /// Upserts a message; silently no-ops if `map_handle` already exists.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` write fails.
    pub async fn upsert(&self, msg: NewMessage) -> Result<(), Error> {
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                let mut stmt = conn.prepare_cached(
                    "INSERT OR IGNORE INTO messages \
                     (map_handle, timestamp_ms, folder, direction, address, address_e164, \
                      status, synced_at, text, outgoing_status) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                )?;
                stmt.execute(params![
                    msg.map_handle,
                    msg.timestamp_ms,
                    msg.folder,
                    msg.direction,
                    msg.address.raw(),
                    msg.address.e164(),
                    msg.status,
                    msg.synced_at,
                    msg.text,
                    msg.outgoing_status,
                ])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns all messages with `timestamp_ms` strictly greater than `after_ms`,
    /// ordered oldest-first.
    ///
    /// Pass `0` to retrieve all stored messages.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` read fails.
    pub async fn messages_since(&self, after_ms: i64) -> Result<Vec<MessageRow>, Error> {
        self.conn()
            .call(
                move |conn: &mut rusqlite::Connection| -> Result<Vec<MessageRow>, rusqlite::Error> {
                    let mut stmt = conn.prepare_cached(
                        "SELECT rowid, map_handle, timestamp_ms, folder, direction, \
                         address, status, synced_at, text, outgoing_status \
                         FROM messages WHERE timestamp_ms > ?1 ORDER BY timestamp_ms",
                    )?;
                    let rows = stmt
                        .query_map([after_ms], |row| {
                            Ok(MessageRow {
                                rowid: row.get(0)?,
                                map_handle: row.get(1)?,
                                timestamp_ms: row.get(2)?,
                                folder: row.get(3)?,
                                direction: row.get(4)?,
                                address: row.get(5)?,
                                status: row.get(6)?,
                                synced_at: row.get(7)?,
                                text: row.get(8)?,
                                outgoing_status: row.get(9)?,
                            })
                        })?
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(rows)
                },
            )
            .await
            .map_err(Error::Connection)
    }

    /// Returns the maximum `timestamp_ms` across all stored messages, or `None` if the store is empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` read fails.
    pub async fn max_timestamp(&self) -> Result<Option<i64>, Error> {
        self.conn()
            .call(|conn: &mut rusqlite::Connection| -> Result<Option<i64>, rusqlite::Error> {
                conn.query_row("SELECT MAX(timestamp_ms) FROM messages", [], |row| row.get(0))
            })
            .await
            .map_err(Error::Connection)
    }

    /// Deletes the message identified by `handle`.
    ///
    /// No-ops silently if the handle is not present.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` write fails.
    pub async fn delete_by_handle(&self, handle: &str) -> Result<(), Error> {
        let handle = handle.to_owned();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                conn.prepare_cached("DELETE FROM messages WHERE map_handle = ?1")?
                    .execute(params![handle])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Updates the `folder` column for the message identified by `handle`.
    ///
    /// No-ops silently if the handle is not present.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` write fails.
    pub async fn update_folder(&self, handle: &str, folder: &str) -> Result<(), Error> {
        let (handle, folder) = (handle.to_owned(), folder.to_owned());
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                conn.prepare_cached("UPDATE messages SET folder = ?1 WHERE map_handle = ?2")?
                    .execute(params![folder, handle])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Updates the `status` column for the message identified by `handle`.
    ///
    /// No-ops silently if the handle is not present.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` write fails.
    pub async fn update_status(&self, handle: &str, status: i32) -> Result<(), Error> {
        let handle = handle.to_owned();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                conn.prepare_cached("UPDATE messages SET status = ?1 WHERE map_handle = ?2")?
                    .execute(params![status, handle])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }
}
