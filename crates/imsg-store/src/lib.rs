//! Encrypted `SQLCipher` message store for imsg.
//!
//! Owns the database connection, schema, migrations, and all query operations.
//! Receives an already-resolved database path and an in-memory key — does not
//! resolve paths and does not talk to the keyring. Introduces no dependency on
//! the protocol crates.

mod cursors;
mod outbox;
mod query;
mod read;
mod row;
mod store;

pub use row::{
    Direction, FolderCursor, FolderSyncStatus, MessageRow, NewMessage, OutboxRow, OutboxStatus,
    OutgoingStatus, ThreadRow, STATUS_READ, STATUS_UNREAD,
};
pub use store::Store;

use thiserror::Error;

/// Database open, async-dispatch, migration, or invalid outbox-transition failures.
#[derive(Debug, Error)]
pub enum Error {
    /// A rusqlite error from opening the database file (bad path, wrong key, etc.).
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// An async task or channel error from `tokio-rusqlite` (call dispatch, task panic, etc.).
    #[error("connection error: {0}")]
    Connection(#[from] tokio_rusqlite::Error),
    /// Schema migrations failed on open; the binary may be a downgrade.
    ///
    /// Inner string preserves the `Box<dyn Error>` message from refinery — the concrete
    /// migration error type is not stable across refinery versions.
    #[error("migration failed: {0}")]
    Migration(String),
    /// `resolve` was called with `OutboxStatus::Queued`, which is not a valid transition target.
    ///
    /// `resolve` advances outbox state; callers must supply `Sending`, `Sent`, `Failed`, or `Unknown`.
    #[error("invalid outbox transition: Queued is not a valid resolve target")]
    InvalidTransition,
}
