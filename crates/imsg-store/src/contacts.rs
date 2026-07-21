//! Cached PBAP contact reads and writes.

use rusqlite::{params, OptionalExtension};

use crate::{Error, Store};

impl Store {
    /// Upserts a contact's display name for `address`, overwriting any existing entry.
    ///
    /// `address` is stored exactly as supplied — callers are responsible for matching
    /// whatever format `messages.address` uses, since [`Store::threads`] joins on equality.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the upsert fails.
    pub async fn upsert_contact(
        &self,
        address: &str,
        display_name: Option<&str>,
    ) -> Result<(), Error> {
        let (address, display_name) = (address.to_owned(), display_name.map(str::to_owned));
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                conn.prepare_cached(
                    "INSERT INTO contacts (address, display_name) VALUES (?1, ?2) \
                     ON CONFLICT(address) DO UPDATE SET display_name = excluded.display_name",
                )?
                .execute(params![address, display_name])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }

    /// Upserts every `(address, display_name)` pair in a single transaction, overwriting any
    /// existing entries. Same per-address contract as [`Store::upsert_contact`]; batched so a
    /// full-phonebook sync is one round trip through the connection actor instead of one per
    /// contact.
    ///
    /// Returns the number of rows written.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the transaction fails.
    pub async fn upsert_contacts(
        &self,
        entries: Vec<(String, Option<String>)>,
    ) -> Result<usize, Error> {
        let count = entries.len();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                let tx = conn.transaction()?;
                {
                    let mut stmt = tx.prepare_cached(
                        "INSERT INTO contacts (address, display_name) VALUES (?1, ?2) \
                         ON CONFLICT(address) DO UPDATE SET display_name = excluded.display_name",
                    )?;
                    for (address, display_name) in &entries {
                        stmt.execute(params![address, display_name])?;
                    }
                }
                tx.commit()
            })
            .await
            .map(|()| count)
            .map_err(Error::Connection)
    }

    /// Returns the cached display name for `address`, or `None` if no contact is cached for
    /// that exact address.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the read fails.
    pub async fn contact_name(&self, address: &str) -> Result<Option<String>, Error> {
        let address = address.to_owned();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| {
                conn.query_row(
                    "SELECT display_name FROM contacts WHERE address = ?1",
                    [address],
                    |row| row.get(0),
                )
                .optional()
                .map(Option::flatten)
            })
            .await
            .map_err(Error::Connection)
    }
}

#[cfg(test)]
mod tests;
