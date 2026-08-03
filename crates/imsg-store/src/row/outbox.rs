use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use strum_macros::{Display, EnumIs, EnumString};

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
