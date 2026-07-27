//! Contact cache reads: single-contact lookups and paginated listings.

use std::collections::HashMap;

use formats::phone::PhoneField;
use rusqlite::OptionalExtension;

use crate::contacts::types::{ContactEntryRow, ContactRow};
use crate::{Error, Store};

const PAGE_ORDER: &str = "ORDER BY display_name COLLATE NOCASE ASC, uid ASC LIMIT ?1 OFFSET ?2";

/// Returns every cached phone number for `uid`, in insertion (vCard) order.
fn phones_of(conn: &rusqlite::Connection, uid: &str) -> rusqlite::Result<Vec<PhoneField>> {
    conn.prepare_cached(
        "SELECT address, address_e164 FROM contact_phones WHERE uid = ?1 ORDER BY rowid ASC",
    )?
    .query_map([uid], |row| Ok(PhoneField::from_parts(row.get(0)?, row.get(1)?)))?
    .collect()
}

/// Returns cached phone numbers for every UID in `uids`, grouped by UID.
fn phones_for(
    conn: &rusqlite::Connection,
    uids: &[String],
) -> rusqlite::Result<HashMap<String, Vec<PhoneField>>> {
    if uids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; uids.len()].join(",");
    let sql = format!(
        "SELECT uid, address, address_e164 FROM contact_phones \
         WHERE uid IN ({placeholders}) ORDER BY rowid ASC"
    );
    let bound: Vec<&dyn rusqlite::ToSql> =
        uids.iter().map(|u| -> &dyn rusqlite::ToSql { u }).collect();
    let mut out: HashMap<String, Vec<PhoneField>> = HashMap::new();
    for row in conn.prepare(&sql)?.query_map(bound.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, PhoneField::from_parts(row.get(1)?, row.get(2)?)))
    })? {
        let (uid, phone) = row?;
        out.entry(uid).or_default().push(phone);
    }
    Ok(out)
}

/// Returns the full contact for `uid`, or `None` if no contact is cached under that UID.
fn fetch_contact(conn: &rusqlite::Connection, uid: &str) -> rusqlite::Result<Option<ContactRow>> {
    let display_name: Option<Option<String>> = conn
        .query_row("SELECT display_name FROM contacts WHERE uid = ?1", [uid], |row| row.get(0))
        .optional()?;
    let Some(display_name) = display_name else { return Ok(None) };
    let phones = phones_of(conn, uid)?;
    Ok(Some(ContactRow { uid: uid.to_owned(), display_name, phones }))
}

impl Store {
    /// Returns the full contact for `uid`, including all cached phone numbers, or `None`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the read fails.
    pub async fn get_contact(&self, uid: &str) -> Result<Option<ContactRow>, Error> {
        let uid = uid.to_owned();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| fetch_contact(conn, &uid))
            .await
            .map_err(Error::Connection)
    }

    /// Returns the full contact that owns phone number `address`, matched on the canonical form
    /// (E.164 when resolved, else raw), or `None` if no cached contact has that number. Pass an
    /// already-canonical address so formatting differences don't cause a miss.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the read fails.
    pub async fn lookup_contact(&self, address: &str) -> Result<Option<ContactRow>, Error> {
        let address = address.to_owned();
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| {
                let uid: Option<String> = conn
                    .query_row(
                        "SELECT uid FROM contact_phones \
                         WHERE COALESCE(address_e164, address) = ?1 LIMIT 1",
                        [&address],
                        |row| row.get(0),
                    )
                    .optional()?;
                uid.map_or(Ok(None), |uid| fetch_contact(conn, &uid))
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns lightweight contact identities (UID + display name, no phone numbers), ordered
    /// by display name then UID, page-limited.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the read fails.
    pub async fn list_contacts(
        &self,
        limit: u16,
        offset: u16,
    ) -> Result<Vec<ContactEntryRow>, Error> {
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| {
                let sql = format!("SELECT uid, display_name FROM contacts {PAGE_ORDER}");
                conn.prepare_cached(&sql)?
                    .query_map(rusqlite::params![i64::from(limit), i64::from(offset)], |row| {
                        Ok(ContactEntryRow { uid: row.get(0)?, display_name: row.get(1)? })
                    })?
                    .collect()
            })
            .await
            .map_err(Error::Connection)
    }

    /// Returns full contacts (with phone numbers), ordered by display name then UID,
    /// page-limited.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Connection`] if the read fails.
    pub async fn all_contacts(&self, limit: u16, offset: u16) -> Result<Vec<ContactRow>, Error> {
        self.conn()
            .call(move |conn: &mut rusqlite::Connection| {
                let sql = format!("SELECT uid, display_name FROM contacts {PAGE_ORDER}");
                let page: Vec<(String, Option<String>)> = conn
                    .prepare_cached(&sql)?
                    .query_map(rusqlite::params![i64::from(limit), i64::from(offset)], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    })?
                    .collect::<Result<_, _>>()?;
                let uids: Vec<String> = page.iter().map(|(uid, _)| uid.clone()).collect();
                let mut phones_by_uid = phones_for(conn, &uids)?;
                Ok(page
                    .into_iter()
                    .map(|(uid, display_name)| {
                        let phones = phones_by_uid.remove(&uid).unwrap_or_default();
                        ContactRow { uid, display_name, phones }
                    })
                    .collect())
            })
            .await
            .map_err(Error::Connection)
    }
}
