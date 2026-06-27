use rusqlite::params;

use crate::{row::MessageRow, Error, NewMessage, OutgoingStatus, Store};

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
                     (map_handle, timestamp_ms, folder, direction, address, \
                      status, synced_at, text, outgoing_status) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                )?;
                stmt.execute(params![
                    msg.map_handle,
                    msg.timestamp_ms,
                    msg.folder,
                    msg.direction,
                    msg.address,
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

    /// Atomically inserts a speculative `messages` row and a linked `outbox` entry.
    ///
    /// Both inserts are wrapped in a single `SQLite` transaction; either both succeed or
    /// neither is written. The `messages` row is created with a placeholder `map_handle`
    /// of the form `"local:{outbox_id}"` which is later overwritten by [`Store::promote_outgoing`]
    /// once the device assigns a real handle.
    ///
    /// Returns `(message_rowid, outbox_id)`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the transaction fails at any step.
    pub async fn enqueue_send(
        &self,
        msg: NewMessage,
        command: &str,
        payload: &str,
        created_at: i64,
    ) -> Result<(i64, i64), Error> {
        let command = command.to_owned();
        let payload = payload.to_owned();
        self.conn()
            .call(move |conn| {
                use crate::row::OutboxStatus;
                let tx = conn.transaction()?;

                // Insert outbox entry first so its rowid can seed the placeholder handle.
                tx.execute(
                    "INSERT INTO outbox \
                     (command, payload, local_message_id, status, created_at) \
                     VALUES (?1, ?2, NULL, ?3, ?4)",
                    params![command, payload, OutboxStatus::Queued, created_at],
                )?;
                let outbox_id = tx.last_insert_rowid();

                // Speculative message handle: unique, identifiable, replaced on push success.
                let placeholder = format!("local:{outbox_id}");
                tx.execute(
                    "INSERT INTO messages \
                     (map_handle, timestamp_ms, folder, direction, address, \
                      status, synced_at, text, outgoing_status) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        placeholder,
                        msg.timestamp_ms,
                        msg.folder,
                        msg.direction,
                        msg.address,
                        msg.status,
                        msg.synced_at,
                        msg.text,
                        msg.outgoing_status,
                    ],
                )?;
                let msg_rowid = tx.last_insert_rowid();

                // Link the outbox entry to its speculative message row.
                tx.execute(
                    "UPDATE outbox SET local_message_id = ?1 WHERE id = ?2",
                    params![msg_rowid, outbox_id],
                )?;

                tx.commit()?;
                Ok((msg_rowid, outbox_id))
            })
            .await
            .map_err(Error::Connection)
    }

    /// Updates `outgoing_status` for the message identified by `handle`.
    ///
    /// Used on push failure or ambiguous outcome. No-ops silently if the handle is absent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the update fails.
    pub async fn update_outgoing_status(
        &self,
        handle: &str,
        status: OutgoingStatus,
    ) -> Result<(), Error> {
        let handle = handle.to_owned();
        self.conn()
            .call(move |conn| {
                conn.prepare_cached(
                    "UPDATE messages SET outgoing_status = ?1 WHERE map_handle = ?2",
                )?
                .execute(params![status, handle])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Renames the placeholder `map_handle` to the device-assigned handle and simultaneously
    /// sets `outgoing_status` to the supplied value.
    ///
    /// Called on push success: `old_handle` is `"local:{outbox_id}"`, `new_handle` is the
    /// real MAP handle returned by the device. No-ops silently if `old_handle` is absent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the update fails.
    pub async fn promote_outgoing(
        &self,
        old_handle: &str,
        new_handle: &str,
        status: OutgoingStatus,
    ) -> Result<(), Error> {
        let (old, new) = (old_handle.to_owned(), new_handle.to_owned());
        self.conn()
            .call(move |conn| {
                conn.prepare_cached(
                    "UPDATE messages SET map_handle = ?1, outgoing_status = ?2 \
                     WHERE map_handle = ?3",
                )?
                .execute(params![new, status, old])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Advances `outgoing_status` from `sent_unconfirmed` to `sent_confirmed` for `handle`.
    ///
    /// Called during Sent-folder backfill reconciliation: when a device Sent message matches
    /// a local row that was speculatively created by [`Store::enqueue_send`], this confirms
    /// the device has the message. No-ops if the row does not exist or has a different status.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the update fails.
    pub async fn reconcile_outgoing(&self, handle: &str) -> Result<(), Error> {
        let handle = handle.to_owned();
        self.conn()
            .call(move |conn| {
                conn.prepare_cached(
                    "UPDATE messages SET outgoing_status = 'sent_confirmed' \
                     WHERE map_handle = ?1 AND outgoing_status = 'sent_unconfirmed'",
                )?
                .execute([handle.as_str()])?;
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

    /// Returns the `last_sync_at` timestamp from the `meta` table, or `None` if never set.
    ///
    /// A `None` result means no backfill has completed; callers should treat the store as empty.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` read fails.
    pub async fn last_sync_at(&self) -> Result<Option<i64>, Error> {
        self.conn()
            .call(|conn: &mut rusqlite::Connection| -> Result<Option<i64>, rusqlite::Error> {
                let mut stmt =
                    conn.prepare_cached("SELECT value FROM meta WHERE key = 'last_sync_at'")?;
                let mut rows = stmt.query([])?;
                rows.next()?.map_or_else(
                    || Ok(None),
                    |row| {
                        row.get::<_, String>(0)?.parse::<i64>().map(Some).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })
                    },
                )
            })
            .await
            .map_err(Error::Connection)
    }

    /// Persists `ms` as the `last_sync_at` anchor in the `meta` table.
    ///
    /// Subsequent calls overwrite the previous value. Callers set this only after
    /// a backfill run completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` write fails.
    pub async fn set_last_sync_at(&self, ms: i64) -> Result<(), Error> {
        self.set_meta("last_sync_at", &ms.to_string()).await
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

    /// Returns the raw text value stored under `key` in the `meta` table, or `None` if absent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the query fails.
    pub async fn get_meta(&self, key: &str) -> Result<Option<String>, Error> {
        let key = key.to_owned();
        self.conn()
            .call(move |conn| {
                let mut stmt = conn.prepare_cached("SELECT value FROM meta WHERE key = ?1")?;
                let mut rows = stmt.query([key.as_str()])?;
                rows.next()?.map_or_else(|| Ok(None), |row| row.get(0).map(Some))
            })
            .await
            .map_err(Error::Connection)
    }

    /// Atomically resolves the outbox entry to `Sent` and promotes the speculative message handle.
    ///
    /// Wraps both updates in a single `SQLite` transaction: `outbox.status → 'sent'` with
    /// `resolved_at = now_ms`, and `messages.map_handle` renamed from `old_handle` to
    /// `new_handle` with `outgoing_status → 'sent_unconfirmed'`. Either both writes
    /// succeed or neither is visible — callers are safe against partial-update inconsistency.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the transaction fails at any step.
    pub async fn complete_send(
        &self,
        outbox_id: i64,
        old_handle: &str,
        new_handle: &str,
        now_ms: i64,
    ) -> Result<(), Error> {
        let (old, new) = (old_handle.to_owned(), new_handle.to_owned());
        self.conn()
            .call(move |conn| {
                use crate::row::{OutboxStatus, OutgoingStatus};
                let tx = conn.transaction()?;
                tx.execute(
                    "UPDATE outbox SET status = ?1, resolved_at = ?2, error = NULL WHERE id = ?3",
                    params![OutboxStatus::Sent, now_ms, outbox_id],
                )?;
                tx.execute(
                    "UPDATE messages SET map_handle = ?1, outgoing_status = ?2 \
                     WHERE map_handle = ?3",
                    params![new, OutgoingStatus::SentUnconfirmed, old],
                )?;
                tx.commit()?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Upserts `value` under `key` in the `meta` table, overwriting any existing entry.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the upsert fails.
    pub async fn set_meta(&self, key: &str, value: &str) -> Result<(), Error> {
        let (key, value) = (key.to_owned(), value.to_owned());
        self.conn()
            .call(move |conn| {
                conn.prepare_cached(
                    "INSERT INTO meta (key, value) VALUES (?1, ?2) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                )?
                .execute(params![key, value])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }
}
