//! Local-store-backed DTOs for the GUI's read path.
//!
//! Distinct from `imsg-ipc`'s `MessageDto`/`ThreadDto`: those mirror the live-device read
//! path (`session::live::models`) and cross the broker socket; these mirror `imsg-store`'s
//! own row types and never leave this process — the GUI opens its own `Store` connection,
//! same as the CLI's local `list`/`get`/`threads`.

use serde::{Deserialize, Serialize};

pub mod contacts;
pub use contacts::{ContactDto, ContactEntryDto};

/// Whether a message was received from the remote or sent by this device.
///
/// Mirrors `store::Direction`; kept as a separate type so a GUI-only frontend concern never
/// forces a wire-shape change on the store's own row type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum Direction {
    /// Received from the remote.
    Received,
    /// Sent by this device.
    Sent,
}

impl From<store::Direction> for Direction {
    fn from(d: store::Direction) -> Self {
        match d {
            store::Direction::Received => Self::Received,
            store::Direction::Sent => Self::Sent,
        }
    }
}

/// Outbox delivery state of a locally-sent message; mirrors `store::OutgoingStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
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

impl From<store::OutgoingStatus> for OutgoingStatus {
    fn from(s: store::OutgoingStatus) -> Self {
        match s {
            store::OutgoingStatus::Queued => Self::Queued,
            store::OutgoingStatus::Sending => Self::Sending,
            store::OutgoingStatus::SentUnconfirmed => Self::SentUnconfirmed,
            store::OutgoingStatus::SentConfirmed => Self::SentConfirmed,
            store::OutgoingStatus::FailedRetryable => Self::FailedRetryable,
            store::OutgoingStatus::FailedPermanent => Self::FailedPermanent,
            store::OutgoingStatus::Unknown => Self::Unknown,
        }
    }
}

/// A single message row from the local store, shaped for the GUI's frontend.
///
/// Omits `rowid` (a `SQLite` implementation detail with no frontend meaning) and the raw
/// `status` integer (collapsed into `read`), so the store's internal row shape can change
/// without forcing a frontend/TS-binding change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct MessageDto {
    /// Opaque MAP message handle.
    pub handle: String,
    /// Milliseconds since Unix epoch.
    pub timestamp_ms: i64,
    /// MAP folder the message resides in (e.g. `telecom/msg/inbox`).
    pub folder: String,
    /// Received vs. sent.
    pub direction: Direction,
    /// Remote phone number or address.
    pub address: String,
    /// `true` unless the store's raw status marks the message unread.
    pub read: bool,
    /// Decoded message body text.
    pub text: String,
    /// Outbox delivery state; `None` for received messages.
    pub outgoing_status: Option<OutgoingStatus>,
}

impl From<&store::MessageRow> for MessageDto {
    fn from(row: &store::MessageRow) -> Self {
        Self {
            handle: row.map_handle.clone(),
            timestamp_ms: row.timestamp_ms,
            folder: row.folder.clone(),
            direction: row.direction.into(),
            address: row.address.clone(),
            read: row.status != store::STATUS_UNREAD,
            text: row.text.clone(),
            outgoing_status: row.outgoing_status.map(Into::into),
        }
    }
}

/// A per-contact conversation thread summary from the local store, shaped for the GUI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ThreadDto {
    /// Contact address.
    pub address: String,
    /// Milliseconds since Unix epoch of the most recent message in this thread.
    pub latest_ms: i64,
    /// Total message count across all folders for this address.
    pub total: i64,
    /// Count of unread received messages.
    pub unread: i64,
    /// Outbox delivery state of the most recent message; `None` when it was received.
    pub latest_outgoing_status: Option<OutgoingStatus>,
    /// Cached PBAP display name for this address, if a synced contact claims it.
    pub contact_name: Option<String>,
}

impl From<&store::ThreadRow> for ThreadDto {
    fn from(row: &store::ThreadRow) -> Self {
        Self {
            address: row.address.clone(),
            latest_ms: row.latest_ms,
            total: row.total,
            unread: row.unread,
            latest_outgoing_status: row.latest_outgoing_status.map(Into::into),
            contact_name: row.contact_name.clone(),
        }
    }
}

/// RFCOMM `BT_SECURITY` policy tier requested from the kernel; mirrors `config::SecurityLevel`.
/// See [`Direction`]/[`OutgoingStatus`] for why this is a separate GUI-facing type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum SecurityLevelDto {
    /// `BT_SECURITY_SDP` — SDP-only traffic, no security.
    Sdp,
    /// `BT_SECURITY_LOW` — no encryption or authentication required.
    Low,
    /// `BT_SECURITY_MEDIUM` — encryption required; no authentication (no MITM protection).
    Medium,
    /// `BT_SECURITY_HIGH` — encryption and authentication required (MITM protection).
    High,
}

impl From<config::SecurityLevel> for SecurityLevelDto {
    fn from(level: config::SecurityLevel) -> Self {
        match level {
            config::SecurityLevel::Sdp => Self::Sdp,
            config::SecurityLevel::Low => Self::Low,
            config::SecurityLevel::Medium => Self::Medium,
            config::SecurityLevel::High => Self::High,
        }
    }
}

impl From<SecurityLevelDto> for config::SecurityLevel {
    fn from(level: SecurityLevelDto) -> Self {
        match level {
            SecurityLevelDto::Sdp => Self::Sdp,
            SecurityLevelDto::Low => Self::Low,
            SecurityLevelDto::Medium => Self::Medium,
            SecurityLevelDto::High => Self::High,
        }
    }
}

/// Resolved local configuration (`imsg config show`'s data), shaped for the GUI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ConfigDto {
    /// Bluetooth MAC address (`XX:XX:XX:XX:XX:XX`).
    pub device_address: String,
    /// RFCOMM channel for the MAP MAS profile.
    pub map_channel: u8,
    /// RFCOMM channel for the PBAP PSE profile.
    pub pbap_channel: u8,
    /// iroh hub node key; `None` until `imsg spoke add` has been run.
    pub hub_node_key: Option<String>,
    /// Configured RFCOMM `BT_SECURITY` requirement for the MAP connect socket; `None` means
    /// imsg makes no explicit request — the kernel/BlueZ default (whatever the existing
    /// pairing/bond negotiated) applies unchanged.
    pub security_level: Option<SecurityLevelDto>,
}

impl From<&config::Config> for ConfigDto {
    fn from(cfg: &config::Config) -> Self {
        Self {
            device_address: cfg.device.address().to_owned(),
            map_channel: cfg.device.map_channel,
            pbap_channel: cfg.device.pbap_channel,
            hub_node_key: cfg.hub.node_key.clone(),
            security_level: cfg.broker.security_level.map(Into::into),
        }
    }
}

/// A Bluetooth device paired on the default adapter, shaped for the GUI's device-picker gate.
///
/// Distinct from `imsg-transport::discover::PairedDevice` so a `specta::Type`/frontend-contract
/// concern never forces a wire-shape change on that crate's domain type — same reasoning as
/// `MessageDto`/`ThreadDto` vs. `imsg-store`'s row types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct PairedDeviceDto {
    /// Bluetooth MAC address (`XX:XX:XX:XX:XX:XX`).
    pub address: String,
    /// The remote-advertised friendly name, if `BlueZ` has one cached.
    pub name: Option<String>,
}

impl From<&transport::discover::PairedDevice> for PairedDeviceDto {
    fn from(device: &transport::discover::PairedDevice) -> Self {
        Self { address: device.address.to_string(), name: device.name.clone() }
    }
}

/// RFCOMM channels resolved over SDP for the MAP and PBAP services.
///
/// A `None` field means the device has no service record for that profile — not a failure. See
/// [`PairedDeviceDto`] for why this isn't `imsg-transport::discover::Channels` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct ChannelsDto {
    /// MAP (Message Access Profile) RFCOMM channel.
    pub map: Option<u8>,
    /// PBAP (Phone Book Access Profile) RFCOMM channel.
    pub pbap: Option<u8>,
}

impl From<transport::discover::Channels> for ChannelsDto {
    fn from(channels: transport::discover::Channels) -> Self {
        Self { map: channels.map, pbap: channels.pbap }
    }
}

#[cfg(test)]
mod tests;
