use std::path::PathBuf;

use secrecy::ExposeSecret;
use secrecy::SecretBox;

use crate::Error;

mod embedded {
    refinery::embed_migrations!("migrations");
}

type MigrationError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Handle to the encrypted `SQLCipher` message store.
///
/// Wraps a single `tokio-rusqlite` connection; all operations are serialized
/// on a dedicated background thread. The connection is unlocked with the
/// caller-supplied key on [`Store::open`] and remains open until dropped.
pub struct Store {
    conn: tokio_rusqlite::Connection,
}

impl Store {
    pub(crate) const fn conn(&self) -> &tokio_rusqlite::Connection {
        &self.conn
    }
}

/// Encodes `key` as the `SQLCipher` hex-blob PRAGMA value `x'<hex>'`.
fn make_key_hex(key: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(67); // "x'" + 64 hex chars + "'"
    s.push_str("x'");
    for byte in key {
        let _ = write!(s, "{byte:02x}");
    }
    s.push('\'');
    s
}

impl Store {
    /// Opens the `SQLCipher` database at `path`, unlocks it with `key`, and
    /// runs any pending migrations before returning.
    ///
    /// `path` must point to a writable location; parent directories must exist.
    /// `key` is consumed and zeroed after the PRAGMA is issued.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Database`] if the file cannot be opened, [`Error::Connection`]
    /// if the key PRAGMA or async dispatch fails, or [`Error::Migration`] if a
    /// pending migration cannot be applied.
    pub async fn open(path: PathBuf, key: SecretBox<[u8; 32]>) -> Result<Self, Error> {
        let conn = tokio_rusqlite::Connection::open(&path).await.map_err(Error::Database)?;

        // Key must be the first operation on the connection; WAL and synchronous follow.
        conn.call(move |conn: &mut rusqlite::Connection| -> Result<(), rusqlite::Error> {
            let key_hex = make_key_hex(key.expose_secret());
            conn.pragma_update(None, "key", key_hex.as_str())?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")
        })
        .await?;

        conn.call(|conn: &mut rusqlite::Connection| -> Result<(), MigrationError> {
            embedded::migrations::runner().run(conn).map(|_| ()).map_err(Into::into)
        })
        .await
        .map_err(|e| Error::Migration(e.to_string()))?;

        Ok(Self { conn })
    }
}
