//! Generic key-value accessors backing the `meta` table.

use rusqlite::params;

use crate::{Error, Store};

impl Store {
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
