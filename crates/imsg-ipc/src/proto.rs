//! Wire types for the broker IPC channel.

use serde::{Deserialize, Serialize};

use crate::rows::{BodyDto, CardEntryDto, ContactDto, MessageDto, SyncReportDto, ThreadDto};
use crate::{Reason, SessionState};

/// Maximum frame size for broker IPC frames, in bytes.
///
/// Enforced identically by the broker (bind side) and the CLI (connect side).
/// Large enough for any realistic SMS body; increase when MMS attachments are brokered.
pub const MAX_FRAME_LEN: usize = 64 * 1024;

/// A single request from a CLI process to the broker.
///
/// Sent as one length-delimited JSON frame. The broker processes one request at a time;
/// concurrent CLI invocations queue in the kernel accept backlog.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum BrokerRequest {
    /// Drain the outbox then backfill MAP messages.
    Sync {
        /// MAP path (e.g. `"TELECOM/MSG/INBOX"`); `None` syncs all four standard folders.
        folder: Option<String>,
    },
    /// Record and push an outgoing SMS to the device; broker handles the full send lifecycle.
    Send {
        /// Recipient phone number.
        number: String,
        /// UTF-8 message body.
        message: String,
    },
    /// Set the deleted flag on the device and remove the message from the local store.
    Delete {
        /// Opaque MAP message handle.
        handle: String,
        /// MAP folder path the handle resides in.
        folder: String,
    },
    /// Run an incremental catch-up sync across all folders (for `list`/`get`/`threads` on the
    /// non-opted-in path). Runs `backfill_catch_up`; no folder scoping.
    Backfill,
    /// Query the device live for a folder listing (non-opted-in `list`). No store write; the
    /// broker answers with [`BrokerResponse::Messages`]. Filters are applied client-side over the
    /// device's fixed listing window.
    ListMessages {
        /// MAP folder name (`inbox`/`sent`/`outbox`/`deleted`); `None` defaults to inbox.
        folder: Option<String>,
        /// Keep only unread messages.
        unread: bool,
        /// Keep only messages whose resolved address equals this value exactly.
        from: Option<String>,
        /// Earliest message datetime as a MAP string (`YYYYMMDDTHHMMSS`); ignored if unparseable.
        since: Option<String>,
        /// Maximum rows after `offset`; `None` keeps the device window.
        limit: Option<u16>,
        /// Rows to skip from the newest-first window.
        offset: u16,
    },
    /// Fetch one message body live by handle (non-opted-in `get`). No store write; the broker
    /// answers with [`BrokerResponse::Body`].
    GetMessage {
        /// Opaque MAP message handle.
        handle: String,
    },
    /// Aggregate live Inbox+Sent listings into per-contact threads (non-opted-in `threads`). No
    /// store write; the broker answers with [`BrokerResponse::Threads`].
    Threads,
    /// Mark a message read on the device only (non-opted-in `get --read`). No store write — the
    /// message is not persisted on this path. The broker answers with [`BrokerResponse::Ok`].
    MarkReadDevice {
        /// Opaque MAP message handle.
        handle: String,
    },
    /// Push an outgoing SMS to the device only (non-opted-in `send`). No store write — fire-and-forget
    /// with no outbox tracking or retry. The broker answers with [`BrokerResponse::Text`].
    SendLive {
        /// Recipient phone number.
        number: String,
        /// UTF-8 message body.
        message: String,
    },
    /// Query the device live for a phonebook listing (`contacts --list`). No store write;
    /// pagination is device-side. The broker answers with [`BrokerResponse::ContactEntries`].
    ListContacts {
        /// Lowercase PBAP phonebook path name (`"pb"`/`"ich"`/`"och"`/`"mch"`/`"cch"`/`"spd"`/
        /// `"fav"`); `None` defaults to `"pb"`.
        path: Option<String>,
        /// Maximum rows after `offset`; `None` keeps the device window.
        limit: Option<u16>,
        /// Rows to skip, in device-reported order.
        offset: u16,
    },
    /// Fetch one contact vCard live by handle (`contacts --get`). No store write. The broker
    /// answers with [`BrokerResponse::Contact`]; an unresolvable handle is a device failure,
    /// not a `None` result — see [`BrokerRequest::LookupContact`] for the has-a-match-or-not case.
    GetContact {
        /// Lowercase PBAP phonebook path name; `None` defaults to `"pb"`.
        path: Option<String>,
        /// Opaque PBAP vCard handle.
        handle: String,
    },
    /// Reverse-look up a contact by phone number live (`contacts --lookup`), then pull its
    /// vCard. No store write. The broker answers with [`BrokerResponse::ContactLookup`].
    LookupContact {
        /// Lowercase PBAP phonebook path name; `None` defaults to `"pb"`.
        path: Option<String>,
        /// Phone number to search for; matching is whatever the device's own search implements.
        number: String,
    },
    /// Pull every contact vCard in a phonebook live (`contacts` with no flags). No store write;
    /// pagination is device-side. The broker answers with [`BrokerResponse::Contacts`].
    PullAllContacts {
        /// Lowercase PBAP phonebook path name; `None` defaults to `"pb"`.
        path: Option<String>,
        /// Maximum rows after `offset`; `None` keeps the device window.
        limit: Option<u16>,
        /// Rows to skip, in device-reported order.
        offset: u16,
    },
    /// Stream MAP notification events; broker sends zero or more [`BrokerResponse::WatchEvent`]
    /// frames until the client closes the connection.
    Watch,
    /// Pull the main phonebook and upsert contact display names into the local contacts cache
    /// (see `session::contacts::sync_contacts`). Broker owns the PBAP connection — local mode
    /// only, no hub/spoke equivalent. Answered with [`BrokerResponse::ContactsSynced`].
    SyncContacts,
    /// Return the broker's current connection state; always answered with one frame.
    Status,
    /// Request a graceful stop. Answered with [`BrokerResponse::Ok`] once accepted, or
    /// [`BrokerResponse::Error`] if this broker instance has no shutdown coordinator (the
    /// ephemeral one-shot broker; only persistent/daemon mode supports this).
    Shutdown,
}

/// A single response frame from the broker.
///
/// For one-shot requests the broker sends exactly one frame. For [`BrokerRequest::Watch`]
/// the broker sends a stream of [`BrokerResponse::WatchEvent`] frames until the client
/// closes the connection.
///
/// Adjacently tagged (`kind`/`data`) so newtype variants carrying a `String` or [`Reason`]
/// serialise — internal tagging cannot represent a newtype wrapping a non-map value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum BrokerResponse {
    /// Operation completed successfully with no textual result.
    Ok,
    /// Operation completed; inner string is the human-readable outcome.
    Text(String),
    /// IPC-plumbing failure (malformed frame, broker shutting down, serialisation). Distinct from
    /// a device/session failure, which is carried by [`BrokerResponse::Failed`].
    Error(String),
    /// A MAP operation or connection failed with a typed, action-oriented reason.
    Failed(Reason),
    /// One MAP notification event; only appears in response to [`BrokerRequest::Watch`].
    WatchEvent(WatchEvent),
    /// Live folder listing rows; sole answer to [`BrokerRequest::ListMessages`].
    Messages(Vec<MessageDto>),
    /// Live per-contact thread summaries; sole answer to [`BrokerRequest::Threads`].
    Threads(Vec<ThreadDto>),
    /// One live message body; sole answer to [`BrokerRequest::GetMessage`].
    Body(BodyDto),
    /// Broker health snapshot; always sent in response to [`BrokerRequest::Status`].
    StatusInfo {
        /// Current MAP session lifecycle state (`Active` means connected).
        state: SessionState,
        /// Bluetooth MAC address of the paired device this broker owns.
        device: String,
        /// `true` when this broker is running in persistent (daemon) mode. Distinguishes it
        /// from an ephemeral one-shot broker that happens to be holding the same socket, which
        /// self-idles and carries no shutdown coordinator.
        persistent: bool,
    },
    /// Contacts cache sync completed; sole answer to [`BrokerRequest::SyncContacts`].
    ContactsSynced {
        /// What the sync did — an untouched cache, or a refresh and what it managed to store.
        report: SyncReportDto,
    },
    /// Live phonebook listing rows; sole answer to [`BrokerRequest::ListContacts`].
    ContactEntries(Vec<CardEntryDto>),
    /// One live contact vCard fetched by handle; sole answer to [`BrokerRequest::GetContact`].
    Contact(ContactDto),
    /// Live reverse-lookup result; `None` when the device reports no match. Sole answer to
    /// [`BrokerRequest::LookupContact`].
    ContactLookup(Option<ContactDto>),
    /// Every live contact vCard in a phonebook; sole answer to
    /// [`BrokerRequest::PullAllContacts`].
    Contacts(Vec<ContactDto>),
}

/// MAP 1.4 MNS notification event type, mirroring `map_core::mns_event::EventType`.
///
/// Serialises as the bare variant-name string (e.g. `"NewMessage"`) — wire-compatible with
/// any consumer that only ever read the raw string this field used to carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum EventType {
    /// A new message arrived in a folder.
    NewMessage,
    /// An outgoing message was delivered to the peer.
    DeliverySuccess,
    /// An outgoing message left the device successfully (delivery to the peer not confirmed).
    SendingSuccess,
    /// An outgoing message failed to be delivered to the peer.
    DeliveryFailure,
    /// An outgoing message failed to leave the device.
    SendingFailure,
    /// A message was deleted from a folder.
    MessageDeleted,
    /// A message moved between folders.
    MessageShift,
    /// Device message memory is full; new messages may not be received.
    MemoryFull,
    /// Device message memory freed up after a prior `MemoryFull`.
    MemoryAvailable,
    /// A message's read/unread status changed on the device.
    ReadStatusChanged,
}

/// A MAP notification event flattened for cross-process transport.
///
/// Mirrors `map_core::mns_event::MnsEvent`; fields absent in the `<event>` XML element
/// are `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct WatchEvent {
    /// The notification's event type.
    pub event_type: EventType,
    /// Opaque MAP message handle; absent for memory-state events.
    pub handle: Option<String>,
    /// Current folder path; absent for memory-state events.
    pub folder: Option<String>,
    /// Previous folder path; present only for `MessageShift` events.
    pub old_folder: Option<String>,
    /// Message type string (e.g. `"SMS_GSM"`); absent for memory-state events.
    pub msg_type: Option<String>,
    /// ISO 8601 basic datetime string (e.g. `"20260624T120000"`); present for `NewMessage`.
    pub datetime: Option<String>,
}

#[cfg(test)]
mod tests;
