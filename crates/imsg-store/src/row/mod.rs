//! Row and enum types for the message/outbox/cursor store domain.

mod cursor;
mod message;
mod outbox;

pub use cursor::{FolderCursor, FolderSyncStatus};
pub use message::{Direction, MessageRow, NewMessage, ThreadRow, STATUS_READ, STATUS_UNREAD};
pub use outbox::{OutboxRow, OutboxStatus, OutgoingStatus};
