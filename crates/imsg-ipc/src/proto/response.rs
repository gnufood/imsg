//! Response side of the broker IPC channel — split out of `super` to keep that module under
//! the size ceiling.

use serde::{Deserialize, Serialize};

use crate::rows::{BodyDto, CardEntryDto, ContactDto, MessageDto, SyncReportDto, ThreadDto};
use crate::{Reason, SessionState};

/// A single response frame from the broker.
///
/// For one-shot requests the broker sends exactly one frame. For [`super::BrokerRequest::Watch`]
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
    /// One MAP notification event; only appears in response to [`super::BrokerRequest::Watch`].
    WatchEvent(WatchEvent),
    /// Live folder listing rows; sole answer to [`super::BrokerRequest::ListMessages`].
    Messages(Vec<MessageDto>),
    /// Live per-contact thread summaries; sole answer to [`super::BrokerRequest::Threads`].
    Threads(Vec<ThreadDto>),
    /// One live message body; sole answer to [`super::BrokerRequest::GetMessage`].
    Body(BodyDto),
    /// Broker health snapshot; always sent in response to [`super::BrokerRequest::Status`].
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
    /// Contacts cache sync completed; sole answer to [`super::BrokerRequest::SyncContacts`].
    ContactsSynced {
        /// What the sync did — an untouched cache, or a refresh and what it managed to store.
        report: SyncReportDto,
    },
    /// Live phonebook listing rows; sole answer to [`super::BrokerRequest::ListContacts`].
    ContactEntries(Vec<CardEntryDto>),
    /// One live contact vCard fetched by handle; sole answer to
    /// [`super::BrokerRequest::GetContact`].
    Contact(ContactDto),
    /// Live reverse-lookup result; `None` when the device reports no match. Sole answer to
    /// [`super::BrokerRequest::LookupContact`].
    ContactLookup(Option<ContactDto>),
    /// Every live contact vCard in a phonebook; sole answer to
    /// [`super::BrokerRequest::PullAllContacts`].
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
