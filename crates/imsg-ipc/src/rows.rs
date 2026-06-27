//! Row DTOs for the live-query IPC path.
//!
//! Serde mirrors of the lean read models in `imsg-session::live`, defined without any `store` or
//! `map_core` dependency so `imsg-ipc` stays serde-only. The broker maps the session models into
//! these; the CLI renders them.

use serde::{Deserialize, Serialize};

/// Message direction relative to the paired device.
///
/// Serde mirror of `session::live::models::Direction`; the broker converts at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Inbound — received from the peer.
    Received,
    /// Outbound — sent from this device.
    Sent,
}

/// One message row from a live `list`, mirroring `session::live::models::LiveMessage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageDto {
    /// Opaque MAP message handle.
    pub handle: String,
    /// Message datetime in epoch milliseconds.
    pub timestamp_ms: i64,
    /// Resolved peer address.
    pub address: String,
    /// MAP folder name (`inbox`/`sent`/`outbox`/`deleted`).
    pub folder: String,
    /// Device-reported read state.
    pub read: bool,
    /// Decoded message body text.
    pub text: String,
}

/// One thread summary from a live `threads`, mirroring `session::live::models::LiveThread`.
///
/// Counts are approximate — aggregated over the device's listing window, not the full corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadDto {
    /// Peer address grouping the thread.
    pub address: String,
    /// Most recent message datetime in epoch milliseconds.
    pub latest_ms: i64,
    /// Total messages seen for this address across listed folders.
    pub total: u32,
    /// Count of unread received messages.
    pub unread: u32,
}

/// One message body from a live `get`, mirroring `session::live::models::LiveBody`.
///
/// Carries no timestamp by design — a bMessage has no datetime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyDto {
    /// Opaque MAP message handle the body was fetched by.
    pub handle: String,
    /// Direction derived from the bMessage folder.
    pub direction: Direction,
    /// Resolved peer address.
    pub address: String,
    /// MAP folder string from the bMessage (e.g. `telecom/msg/inbox`).
    pub folder: String,
    /// Device-reported read state.
    pub read: bool,
    /// Decoded message body text.
    pub text: String,
}
