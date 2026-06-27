//! Read queries: point lookup, filtered list, and thread aggregation.

use rusqlite::OptionalExtension as _;

use crate::{
    row::{MessageRow, ThreadRow, STATUS_UNREAD},
    Error, OutgoingStatus, Store,
};

/// Maps a `messages` result row (columns 0–9) to a [`MessageRow`].
///
/// Column order must match every `SELECT` that uses this mapper:
/// `rowid, map_handle, timestamp_ms, folder, direction, address, status, synced_at, text, outgoing_status`.
fn map_msg_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRow> {
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
}

/// Builds the SQL and bound parameters for [`Store::list_messages`].
///
/// Always appends `ORDER BY timestamp_ms DESC LIMIT ? OFFSET ?`; `limit` and `offset` are the
/// final two parameters. Intended to be called inside a `tokio_rusqlite` closure so the
/// returned `Vec<Box<dyn ToSql>>` does not need to be `Send`.
fn build_list_query(
    folder: Option<String>,
    unread_only: bool,
    from: Option<String>,
    since_ms: Option<i64>,
    limit: u16,
    offset: u16,
) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut sql = String::from(
        "SELECT rowid, map_handle, timestamp_ms, folder, direction, address, \
         status, synced_at, text, outgoing_status FROM messages WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(6);
    if let Some(f) = folder {
        sql.push_str(" AND folder = ?");
        params.push(Box::new(f));
    }
    if unread_only {
        sql.push_str(" AND status = ?");
        params.push(Box::new(STATUS_UNREAD));
    }
    if let Some(addr) = from {
        sql.push_str(" AND address = ?");
        params.push(Box::new(addr));
    }
    if let Some(since) = since_ms {
        sql.push_str(" AND timestamp_ms >= ?");
        params.push(Box::new(since));
    }
    sql.push_str(" ORDER BY timestamp_ms DESC LIMIT ? OFFSET ?");
    params.push(Box::new(i64::from(limit)));
    params.push(Box::new(i64::from(offset)));
    (sql, params)
}

impl Store {
    /// Returns the message with the given `map_handle`, or `None` if absent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` read fails.
    pub async fn get_by_handle(&self, handle: &str) -> Result<Option<MessageRow>, Error> {
        let handle = handle.to_owned();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| {
                conn.prepare_cached(
                    "SELECT rowid, map_handle, timestamp_ms, folder, direction, address, \
                     status, synced_at, text, outgoing_status \
                     FROM messages WHERE map_handle = ?1 LIMIT 1",
                )?
                .query_row([handle.as_str()], map_msg_row)
                .optional()
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns messages matching all supplied criteria, newest-first.
    ///
    /// `folder` restricts to a single folder leaf (`"inbox"`, `"sent"`, etc.); `None` searches
    /// all folders. `unread_only` adds `status = 0`. `from` matches `address` exactly.
    /// `since_ms` is an inclusive lower bound on `timestamp_ms`. `limit` and `offset` page the
    /// result; pass `1024` and `0` for the default first page.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` read fails.
    pub async fn list_messages(
        &self,
        folder: Option<&str>,
        unread_only: bool,
        from: Option<&str>,
        since_ms: Option<i64>,
        limit: u16,
        offset: u16,
    ) -> Result<Vec<MessageRow>, Error> {
        let folder = folder.map(str::to_owned);
        let from = from.map(str::to_owned);
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| {
                let (sql, params) =
                    build_list_query(folder, unread_only, from, since_ms, limit, offset);
                let param_refs: Vec<&dyn rusqlite::ToSql> =
                    params.iter().map(std::convert::AsRef::as_ref).collect();
                let mut stmt = conn.prepare_cached(&sql)?;
                let rows = stmt.query_map(param_refs.as_slice(), map_msg_row)?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns a per-address thread summary, most-recent-first.
    ///
    /// Groups all stored messages by `address`, counting total messages and unread received
    /// messages (`status = 0`, `direction = 0`). Rows with an empty address are excluded.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the async dispatch or underlying `SQLite` read fails.
    pub async fn threads(&self) -> Result<Vec<ThreadRow>, Error> {
        self.conn()
            .call(|conn: &mut rusqlite::Connection| {
                // Correlated subquery for latest_outgoing_status is efficient because
                // idx_messages_address_time covers (address, timestamp_ms DESC).
                let mut stmt = conn.prepare_cached(
                    "SELECT m.address, \
                            MAX(m.timestamp_ms) AS latest_ms, \
                            COUNT(*) AS total, \
                            SUM(CASE WHEN m.status = 0 AND m.direction = 0 THEN 1 ELSE 0 END) \
                                AS unread, \
                            (SELECT m2.outgoing_status FROM messages m2 \
                             WHERE m2.address = m.address \
                             ORDER BY m2.timestamp_ms DESC LIMIT 1) AS latest_outgoing_status \
                     FROM messages m WHERE m.address != '' \
                     GROUP BY m.address ORDER BY latest_ms DESC",
                )?;
                let rows = stmt.query_map([], |row| {
                    Ok(ThreadRow {
                        address: row.get(0)?,
                        latest_ms: row.get(1)?,
                        total: row.get(2)?,
                        unread: row.get(3)?,
                        latest_outgoing_status: row
                            .get::<_, Option<String>>(4)?
                            .and_then(|s| s.parse::<OutgoingStatus>().ok()),
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .await
            .map_err(Error::Connection)
    }
}
