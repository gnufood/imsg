use formats::phone::PhoneField;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use super::outbox::OutgoingStatus;

/// Raw `status` column value for an unread message.
pub const STATUS_UNREAD: i32 = 0;

/// Raw `status` column value for a read message.
pub const STATUS_READ: i32 = 1;

/// Whether a message was received from the remote or sent by the local device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Inbound — received from the remote device.
    Received = 0,
    /// Outbound — sent from this device.
    Sent = 1,
}

impl ToSql for Direction {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let v: i64 = match self {
            Self::Received => 0,
            Self::Sent => 1,
        };
        Ok(ToSqlOutput::from(v))
    }
}

impl FromSql for Direction {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match i64::column_result(value)? {
            0 => Ok(Self::Received),
            1 => Ok(Self::Sent),
            v => Err(FromSqlError::OutOfRange(v)),
        }
    }
}

/// A message row as it exists in the store, including the auto-assigned rowid.
#[derive(Debug, Clone)]
pub struct MessageRow {
    /// Auto-assigned store rowid; monotonically increasing within this database.
    pub rowid: i64,
    /// MAP protocol message handle identifying the message within the remote folder.
    pub map_handle: String,
    /// Milliseconds since Unix epoch; used for ordering and catch-up queries.
    pub timestamp_ms: i64,
    /// MAP folder the message resides in (e.g. `telecom/msg/inbox`).
    pub folder: String,
    /// Received vs. sent; persisted as 0/1.
    pub direction: Direction,
    /// Remote phone number or address associated with the message.
    pub address: String,
    /// Raw MAP message status integer; interpretation is caller-defined.
    pub status: i32,
    /// Milliseconds since Unix epoch when this message was written to the store.
    pub synced_at: i64,
    /// Decoded message body text.
    pub text: String,
    /// Outgoing delivery state; `None` for all received messages and for sent messages
    /// that pre-date the Phase 4 outbox. Non-`None` only on rows with `direction = Sent`
    /// created via [`crate::Store::enqueue_send`].
    pub outgoing_status: Option<OutgoingStatus>,
}

/// A per-contact conversation thread summary returned by [`crate::Store::threads`].
///
/// Covers all stored messages for a given `address`, sorted by the most recent message
/// timestamp. `total` and `unread` are `i64` to match `SQLite` aggregate return types.
#[derive(Debug, Clone)]
pub struct ThreadRow {
    /// Contact address; always non-empty (empty-address messages are excluded by the query).
    pub address: String,
    /// Milliseconds since Unix epoch of the most recent message in this thread.
    pub latest_ms: i64,
    /// Total message count across all folders for this address.
    pub total: i64,
    /// Count of unread received messages (`status = 0`, `direction = Received`).
    pub unread: i64,
    /// Outgoing delivery state of the most recent message in this thread; `None` when the
    /// latest message is received or was synced before Phase 4.
    pub latest_outgoing_status: Option<OutgoingStatus>,
    /// Cached PBAP display name for `address`, joined from `contacts` via `contact_phones`;
    /// `None` if no cached contact has this exact address (match is a raw string equality,
    /// not phone-normalized).
    pub contact_name: Option<String>,
}

/// A message to be inserted; the store auto-assigns `rowid` on insert. `synced_at` is a field
/// here (unlike `MessageRow`, which reads it back from the row) because the caller controls it.
#[derive(Debug, Clone)]
pub struct NewMessage {
    /// MAP protocol message handle identifying the message within the remote folder.
    pub map_handle: String,
    /// Milliseconds since Unix epoch.
    pub timestamp_ms: i64,
    /// MAP folder the message resides in (e.g. `telecom/msg/inbox`).
    pub folder: String,
    /// Whether the message was received or sent.
    pub direction: Direction,
    /// Remote phone number, carrying the raw device-reported form and, when resolvable, its
    /// E.164 canonical form. Normalised at the session ingress boundary before construction.
    pub address: PhoneField,
    /// Raw MAP message status integer; interpretation is caller-defined.
    pub status: i32,
    /// Milliseconds since Unix epoch when this sync run fetched the message.
    pub synced_at: i64,
    /// Decoded message body text.
    pub text: String,
    /// Outgoing delivery state; `None` for received messages and for sync-ingested sent messages.
    /// Set to `Some(OutgoingStatus::Queued)` only for speculative rows created by
    /// [`crate::Store::enqueue_send`].
    pub outgoing_status: Option<OutgoingStatus>,
}
