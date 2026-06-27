//! Wire types for the broker IPC channel.

use serde::{Deserialize, Serialize};

use crate::rows::{BodyDto, MessageDto, ThreadDto};
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
        /// Message body text.
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
        /// Message body text.
        message: String,
    },
    /// Stream MAP notification events; broker sends zero or more [`BrokerResponse::WatchEvent`]
    /// frames until the client closes the connection.
    Watch,
    /// Return the broker's current connection state; always answered with one frame.
    Status,
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
    },
}

/// A MAP notification event flattened for cross-process transport.
///
/// Mirrors `map_core::mns_event::MnsEvent`; fields absent in the `<event>` XML element
/// are `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// Canonical MAP 1.4 event type string (e.g. `"NewMessage"`, `"DeliverySuccess"`).
    ///
    /// Matches the `type` attribute of the `<event>` XML element exactly; stable across
    /// crate versions and Rust releases.
    pub event_type: String,
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
mod tests {
    use super::*;
    use crate::Direction;

    /// Regression: internally-tagged newtype-of-`String` variants fail to serialise. Adjacent
    /// tagging fixes it, so `Error`/`Text`/`Failed` frames must round-trip.
    #[test]
    fn newtype_response_variants_roundtrip() -> Result<(), serde_json::Error> {
        let cases = [
            BrokerResponse::Ok,
            BrokerResponse::Text("listed 3".into()),
            BrokerResponse::Error("broker shutting down".into()),
            BrokerResponse::Failed(Reason::DeviceUnreachable),
            BrokerResponse::Failed(Reason::OperationFailed("handle 7".into())),
        ];
        for resp in cases {
            let json = serde_json::to_string(&resp)?;
            let back: BrokerResponse = serde_json::from_str(&json)?;
            assert_eq!(format!("{resp:?}"), format!("{back:?}"));
        }
        Ok(())
    }

    #[test]
    fn live_data_response_variants_roundtrip() -> Result<(), serde_json::Error> {
        let cases = [
            BrokerResponse::Messages(vec![MessageDto {
                handle: "7".into(),
                timestamp_ms: 1_700_000_000_000,
                address: "+15550001".into(),
                folder: "inbox".into(),
                read: false,
                text: "hi".into(),
            }]),
            BrokerResponse::Threads(vec![ThreadDto {
                address: "+15550001".into(),
                latest_ms: 1_700_000_000_000,
                total: 4,
                unread: 1,
            }]),
            BrokerResponse::Body(BodyDto {
                handle: "7".into(),
                direction: Direction::Received,
                address: "+15550001".into(),
                folder: "telecom/msg/inbox".into(),
                read: true,
                text: "hi".into(),
            }),
        ];
        for resp in cases {
            let json = serde_json::to_string(&resp)?;
            let back: BrokerResponse = serde_json::from_str(&json)?;
            assert_eq!(format!("{resp:?}"), format!("{back:?}"));
        }
        Ok(())
    }

    #[test]
    fn live_request_variants_roundtrip() -> Result<(), serde_json::Error> {
        let cases = [
            BrokerRequest::ListMessages {
                folder: Some("sent".into()),
                unread: true,
                from: Some("+15550001".into()),
                since: None,
                limit: Some(20),
                offset: 0,
            },
            BrokerRequest::GetMessage { handle: "7".into() },
            BrokerRequest::Threads,
            BrokerRequest::MarkReadDevice { handle: "7".into() },
            BrokerRequest::SendLive { number: "+15550001".into(), message: "hi".into() },
        ];
        for req in cases {
            let json = serde_json::to_string(&req)?;
            let back: BrokerRequest = serde_json::from_str(&json)?;
            assert_eq!(format!("{req:?}"), format!("{back:?}"));
        }
        Ok(())
    }

    #[test]
    fn status_info_carries_state() -> Result<(), serde_json::Error> {
        let resp = BrokerResponse::StatusInfo {
            state: SessionState::Reconnecting,
            device: "AA:BB:CC:DD:EE:FF".into(),
        };
        let json = serde_json::to_string(&resp)?;
        let back: BrokerResponse = serde_json::from_str(&json)?;
        assert!(matches!(
            back,
            BrokerResponse::StatusInfo { state: SessionState::Reconnecting, .. }
        ));
        Ok(())
    }
}
