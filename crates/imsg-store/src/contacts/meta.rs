//! PBAP phonebook identity/version watermark, cached in the generic `meta` table.

use rusqlite::{params, OptionalExtension as _};

use crate::contacts::types::PbapMeta;
use crate::{Error, Store};

const DATABASE_ID_KEY: &str = "pbap_database_id";
const PRIMARY_VERSION_KEY: &str = "pbap_primary_version";
const SECONDARY_VERSION_KEY: &str = "pbap_secondary_version";
const CONTACTS_SYNCED_AT_KEY: &str = "contacts_synced_at";

impl Store {
    /// Returns the cached PBAP phonebook identity/version watermark; fields are `None` if
    /// never set.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the read fails.
    pub async fn pbap_meta(&self) -> Result<PbapMeta, Error> {
        self.conn()
            .call(|conn: &mut rusqlite::Connection| -> rusqlite::Result<PbapMeta> {
                let mut meta = PbapMeta::default();
                let mut stmt =
                    conn.prepare_cached("SELECT key, value FROM meta WHERE key IN (?1, ?2, ?3)")?;
                let rows = stmt.query_map(
                    params![DATABASE_ID_KEY, PRIMARY_VERSION_KEY, SECONDARY_VERSION_KEY],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )?;
                for row in rows {
                    let (key, value) = row?;
                    match key.as_str() {
                        DATABASE_ID_KEY => meta.database_id = Some(value),
                        PRIMARY_VERSION_KEY => meta.primary_version = Some(value),
                        SECONDARY_VERSION_KEY => meta.secondary_version = Some(value),
                        _ => {}
                    }
                }
                Ok(meta)
            })
            .await
            .map_err(Error::Connection)
    }

    /// Persists the PBAP phonebook identity/version watermark. A `None` field in `meta` leaves
    /// the previously cached value untouched.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the write fails.
    pub async fn set_pbap_meta(&self, meta: &PbapMeta) -> Result<(), Error> {
        let meta = meta.clone();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                let tx = conn.transaction()?;
                {
                    let mut stmt = tx.prepare_cached(
                        "INSERT INTO meta (key, value) VALUES (?1, ?2) \
                         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    )?;
                    for (key, value) in [
                        (DATABASE_ID_KEY, &meta.database_id),
                        (PRIMARY_VERSION_KEY, &meta.primary_version),
                        (SECONDARY_VERSION_KEY, &meta.secondary_version),
                    ] {
                        if let Some(value) = value {
                            stmt.execute(params![key, value])?;
                        }
                    }
                }
                tx.commit()
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns the timestamp of the last successful `sync_contacts` run, or `None` if contacts
    /// have never been synced.
    ///
    /// Freshness signal for the `contacts` command's read paths — a separate domain from the
    /// message-sync `folder_cursors` anchor (see [`Store::latest_sync_at`]).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the query fails.
    pub async fn contacts_synced_at(&self) -> Result<Option<i64>, Error> {
        self.conn()
            .call(|conn: &mut rusqlite::Connection| -> rusqlite::Result<Option<i64>> {
                let value: Option<String> = conn
                    .query_row(
                        "SELECT value FROM meta WHERE key = ?1",
                        params![CONTACTS_SYNCED_AT_KEY],
                        |row| row.get(0),
                    )
                    .optional()?;
                value
                    .map(|v| {
                        v.parse::<i64>().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })
                    })
                    .transpose()
            })
            .await
            .map_err(Error::Connection)
    }

    /// Persists `ms` as the contacts-sync freshness anchor. Subsequent calls overwrite the
    /// previous value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the write fails.
    pub async fn set_contacts_synced_at(&self, ms: i64) -> Result<(), Error> {
        self.set_meta(CONTACTS_SYNCED_AT_KEY, &ms.to_string()).await
    }
}
