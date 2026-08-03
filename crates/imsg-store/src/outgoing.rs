//! Outgoing-message lifecycle: speculative send rows, handle promotion, and reconciliation
//! against the device Sent folder.

use rusqlite::params;

use crate::{row::OutboxStatus, Error, NewMessage, OutgoingStatus, Store};

impl Store {
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
                     (map_handle, timestamp_ms, folder, direction, address, address_e164, \
                      status, synced_at, text, outgoing_status) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        placeholder,
                        msg.timestamp_ms,
                        msg.folder,
                        msg.direction,
                        msg.address.raw(),
                        msg.address.e164(),
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
}

#[cfg(test)]
mod tests;
