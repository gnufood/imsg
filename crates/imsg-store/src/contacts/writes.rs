//! Contact cache writes: batch upsert and full wipe.

use rusqlite::params;

use crate::contacts::types::NewContact;
use crate::{Error, Store};

impl Store {
    /// Upserts every contact in `contacts` — display name and full phone set — in a single
    /// transaction. See [`NewContact`] for phone-replacement semantics. If a phone number is
    /// currently owned by a different contact (the device reported it on two cards), the
    /// later entry in `contacts` steals ownership — `contact_phones.address` is unique.
    ///
    /// Returns the number of contacts written.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the transaction fails.
    pub async fn upsert_contacts(&self, contacts: Vec<NewContact>) -> Result<usize, Error> {
        let count = contacts.len();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                let tx = conn.transaction()?;
                {
                    let mut upsert = tx.prepare_cached(
                        "INSERT INTO contacts (uid, display_name) VALUES (?1, ?2) \
                         ON CONFLICT(uid) DO UPDATE SET display_name = excluded.display_name",
                    )?;
                    let mut clear_phones =
                        tx.prepare_cached("DELETE FROM contact_phones WHERE uid = ?1")?;
                    let mut insert_phone = tx.prepare_cached(
                        "INSERT INTO contact_phones (address, address_e164, uid) VALUES (?1, ?2, ?3) \
                         ON CONFLICT(address) DO UPDATE SET \
                             uid = excluded.uid, address_e164 = excluded.address_e164",
                    )?;
                    for c in &contacts {
                        upsert.execute(params![c.uid, c.display_name])?;
                        clear_phones.execute(params![c.uid])?;
                        for phone in &c.phones {
                            insert_phone.execute(params![phone.raw(), phone.e164(), c.uid])?;
                        }
                    }
                }
                tx.commit()
            })
            .await
            .map(|()| count)
            .map_err(Error::Connection)
    }

    /// Deletes every cached contact and phone number. Used when the device reports a changed
    /// `DatabaseIdentifier`, meaning previously cached UIDs may no longer be valid.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the delete fails.
    pub async fn clear_contacts(&self) -> Result<(), Error> {
        self.conn()
            .call(|conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
                conn.execute("DELETE FROM contact_phones", [])?;
                conn.execute("DELETE FROM contacts", [])?;
                Ok(())
            })
            .await
            .map_err(Error::Connection)
    }
}
