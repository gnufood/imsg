use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use strum_macros::{Display, EnumIs, EnumString};

/// Raw `status` column value for an unread message.
pub const STATUS_UNREAD: i32 = 0;

/// Raw `status` column value for a read message.
pub const STATUS_READ: i32 = 1;

/// Whether a message was received from the remote or sent by the local device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Stored as 0.
    Received = 0,
    /// Stored as 1.
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
    /// created via [`Store::enqueue_send`].
    pub outgoing_status: Option<OutgoingStatus>,
}

/// A per-contact conversation thread summary returned by [`Store::threads`].
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
}

/// A message to be inserted; rowid and `synced_at` are assigned by the caller.
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
    /// Remote phone number or address associated with the message.
    pub address: String,
    /// Raw MAP message status integer; interpretation is caller-defined.
    pub status: i32,
    /// Milliseconds since Unix epoch when this sync run fetched the message.
    pub synced_at: i64,
    /// Decoded message body text.
    pub text: String,
    /// Outgoing delivery state; `None` for received messages and for sync-ingested sent messages.
    /// Set to `Some(OutgoingStatus::Queued)` only for speculative rows created by
    /// [`Store::enqueue_send`].
    pub outgoing_status: Option<OutgoingStatus>,
}

/// Lifecycle state of a row in the `outbox` table.
///
/// Progresses `queued` → `sending` → `sent` | `failed` | `unknown`. `unknown` is set when
/// the connection drops after a push was initiated but before acknowledgement was received;
/// reconciliation against the device Sent folder is required to resolve it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIs)]
#[strum(serialize_all = "snake_case")]
pub enum OutboxStatus {
    /// Waiting for the sync worker to attempt the push.
    Queued,
    /// Push initiated; awaiting device acknowledgement.
    Sending,
    /// Device acknowledged the push successfully.
    Sent,
    /// Push failed with a definitive error; will not be retried automatically.
    Failed,
    /// Connection dropped mid-push; outcome requires reconciliation to determine.
    Unknown,
}

impl ToSql for OutboxStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for OutboxStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = String::column_result(value)?;
        s.parse().map_err(|_| FromSqlError::InvalidType)
    }
}

/// Fine-grained state of an outgoing row in the `messages` table.
///
/// `NULL` for all received messages. Progresses from `queued` toward `sent_confirmed`
/// or a terminal failure state. `unknown` requires reconciliation against the device
/// Sent folder before the outcome can be recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIs)]
#[strum(serialize_all = "snake_case")]
pub enum OutgoingStatus {
    /// Outbox entry created; push not yet attempted.
    Queued,
    /// Push in progress.
    Sending,
    /// Device accepted the push; not yet confirmed by the Sent folder.
    SentUnconfirmed,
    /// Confirmed present in the device Sent folder via reconciliation.
    SentConfirmed,
    /// Push failed with a transient error; a retry is warranted.
    FailedRetryable,
    /// Push failed with a permanent error; no retry will be attempted.
    FailedPermanent,
    /// Connection dropped mid-push; outcome requires reconciliation to determine.
    Unknown,
}

impl ToSql for OutgoingStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for OutgoingStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = String::column_result(value)?;
        s.parse().map_err(|_| FromSqlError::InvalidType)
    }
}

/// Completion state stored in `folder_cursors.sync_status`.
///
/// `never` is the initial value before any sync attempt on a given folder.
/// `complete` means the last sync finished without error and `highest_ts` is reliable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIs)]
#[strum(serialize_all = "snake_case")]
pub enum FolderSyncStatus {
    /// No sync has been attempted for this folder yet.
    Never,
    /// A sync is currently in progress.
    Syncing,
    /// Last sync completed successfully.
    Complete,
    /// Last sync ended with an error; `highest_ts` reflects the last successful boundary.
    Failed,
}

impl ToSql for FolderSyncStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for FolderSyncStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = String::column_result(value)?;
        s.parse().map_err(|_| FromSqlError::InvalidType)
    }
}

/// A row from the `outbox` table representing one pending or resolved outgoing intent.
#[derive(Debug, Clone)]
pub struct OutboxRow {
    /// Auto-assigned primary key.
    pub id: i64,
    /// Verb identifying the outgoing operation (e.g. `"send_sms"`).
    pub command: String,
    /// Serialised parameters for the command.
    pub payload: String,
    /// Rowid of the speculative `messages` row created alongside this entry, if any.
    pub local_message_id: Option<i64>,
    /// Current lifecycle state of this outbox entry.
    pub status: OutboxStatus,
    /// Milliseconds since Unix epoch when this entry was created.
    pub created_at: i64,
    /// Milliseconds since Unix epoch of the most recent push attempt, or `None` if not yet tried.
    pub attempted_at: Option<i64>,
    /// Milliseconds since Unix epoch when the entry reached a terminal state, or `None` if active.
    pub resolved_at: Option<i64>,
    /// Last failure description when `status` is `failed` or `unknown`; `None` otherwise.
    pub error: Option<String>,
}

/// Per-folder sync cursor stored in `folder_cursors`.
///
/// Replaces the single global `last_sync_at` anchor in the `meta` table. Each folder
/// tracks its own progress so a partial sync on one folder never corrupts another.
#[derive(Debug, Clone)]
pub struct FolderCursor {
    /// MAP folder leaf (e.g. `"inbox"`, `"sent"`).
    pub folder: String,
    /// Milliseconds since Unix epoch of the last completed sync run for this folder.
    pub last_sync_at: i64,
    /// Highest `timestamp_ms` seen during the last sync; used as the next pull boundary.
    pub highest_ts: i64,
    /// Whether the last sync completed, is in progress, or failed.
    pub sync_status: FolderSyncStatus,
}
