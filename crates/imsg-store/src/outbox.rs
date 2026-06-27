//! Outbox store operations: enqueue, resolve, and drain pending outgoing intents.

use rusqlite::params;

use crate::{
    row::{OutboxRow, OutboxStatus},
    Error, Store,
};

/// Classifies an `OutboxStatus` for the `resolve` update path.
///
/// Only valid transition targets are accepted; `Queued` is not a valid target
/// because `resolve` advances state — it never resets to the initial state.
enum Transition {
    /// Terminal — sets `resolved_at`.
    Terminal,
    /// In-progress — sets `attempted_at`.
    InProgress,
}

/// Maps a rusqlite row (columns 0–8) to an [`OutboxRow`].
///
/// Column order must match every `SELECT` that feeds this function:
/// `id, command, payload, local_message_id, status, created_at, attempted_at, resolved_at, error`.
fn map_outbox_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OutboxRow> {
    Ok(OutboxRow {
        id: row.get(0)?,
        command: row.get(1)?,
        payload: row.get(2)?,
        local_message_id: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        attempted_at: row.get(6)?,
        resolved_at: row.get(7)?,
        error: row.get(8)?,
    })
}

impl Store {
    /// Inserts a new outbox entry with `status = queued` and returns its auto-assigned `id`.
    ///
    /// `created_at` must be milliseconds since Unix epoch; supply via the caller's clock
    /// so that tests can control the value without touching wall time.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the insert fails.
    pub async fn enqueue(
        &self,
        command: &str,
        payload: &str,
        local_message_id: Option<i64>,
        created_at: i64,
    ) -> Result<i64, Error> {
        let command = command.to_owned();
        let payload = payload.to_owned();
        self.conn()
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO outbox \
                     (command, payload, local_message_id, status, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![command, payload, local_message_id, OutboxStatus::Queued, created_at],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Advances the lifecycle state of outbox entry `id`.
    ///
    /// Terminal states (`sent`, `failed`, `unknown`) set `resolved_at = now_ms`.
    /// The non-terminal `sending` state sets `attempted_at = now_ms` instead.
    /// `error` is stored verbatim and should be `None` for successful transitions.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the update fails.
    /// Returns [`Error::InvalidTransition`] if `status` is `Queued`, which is not a
    /// valid transition target.
    pub async fn resolve(
        &self,
        id: i64,
        status: OutboxStatus,
        now_ms: i64,
        error: Option<String>,
    ) -> Result<(), Error> {
        let transition = match status {
            OutboxStatus::Queued => return Err(Error::InvalidTransition),
            OutboxStatus::Sending => Transition::InProgress,
            OutboxStatus::Sent | OutboxStatus::Failed | OutboxStatus::Unknown => {
                Transition::Terminal
            }
        };
        self.conn()
            .call(move |conn| {
                match transition {
                    Transition::Terminal => conn.execute(
                        "UPDATE outbox SET status = ?1, resolved_at = ?2, error = ?3 \
                         WHERE id = ?4",
                        params![status, now_ms, error, id],
                    )?,
                    Transition::InProgress => conn.execute(
                        "UPDATE outbox SET status = ?1, attempted_at = ?2, error = ?3 \
                         WHERE id = ?4",
                        params![status, now_ms, error, id],
                    )?,
                };
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns all outbox entries with `status = queued`, ordered oldest first.
    ///
    /// The sync worker calls this on startup and after each reconnect to drain the queue.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the query fails.
    pub async fn pending(&self) -> Result<Vec<OutboxRow>, Error> {
        self.conn()
            .call(|conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, command, payload, local_message_id, status, \
                     created_at, attempted_at, resolved_at, error \
                     FROM outbox WHERE status = ?1 ORDER BY created_at ASC",
                )?;
                let rows = stmt.query_map([OutboxStatus::Queued], map_outbox_row)?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .await
            .map_err(Error::Connection)
    }
}
