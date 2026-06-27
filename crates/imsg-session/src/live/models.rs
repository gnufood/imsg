//! Lean read models for the live-query path.
//!
//! Device-derived message/thread/body shapes carrying no store-only fields (no `rowid`,
//! `synced_at`, or `outgoing_status`, which have no device source). Only these models leave
//! `imsg-session`; the `map_core` protocol types they are normalized from do not.

/// Message direction relative to this device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Inbound — received from the peer.
    Received,
    /// Outbound — sent from this device.
    Sent,
}

/// One message from a live folder listing, body text fetched on demand.
///
/// `list`-path carrier. `timestamp_ms` is always present (taken from the listing entry, falling
/// back to fetch time only on a malformed datetime). `read` is the device-reported read state.
/// Carries no direction — the list renderer does not use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveMessage {
    /// Opaque MAP message handle.
    pub handle: String,
    /// Message datetime in epoch milliseconds.
    pub timestamp_ms: i64,
    /// Resolved peer address: sender for received, recipient for sent.
    pub address: String,
    /// MAP folder name (`inbox`/`sent`/`outbox`/`deleted`).
    pub folder: String,
    /// Device-reported read state.
    pub read: bool,
    /// Decoded message body text.
    pub text: String,
}

/// Per-contact thread summary aggregated from live folder listings.
///
/// Counts are approximate: aggregated over the device's bounded listing window, not the full
/// corpus, so they differ from the store's full-corpus `GROUP BY`. No delivery badge — outgoing
/// status is outbox-only with no device source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveThread {
    /// Peer address grouping the thread; never empty (empty-address entries are dropped).
    pub address: String,
    /// Most recent message datetime in epoch milliseconds across listed folders.
    pub latest_ms: i64,
    /// Total messages seen for this address across listed folders.
    pub total: u32,
    /// Count of unread received messages (`!sent && !read`).
    pub unread: u32,
}

/// One message body fetched live by handle.
///
/// `get`-path carrier. Structurally carries no timestamp — a `BMessage` has no datetime — and
/// carries `direction` because the `get` renderer prints a `From:`/`To:` label from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveBody {
    /// Opaque MAP message handle the body was fetched by.
    pub handle: String,
    /// Direction derived from the bMessage folder.
    pub direction: Direction,
    /// Resolved peer address: originator for received, first recipient for sent.
    pub address: String,
    /// MAP folder string from the bMessage (e.g. `telecom/msg/inbox`).
    pub folder: String,
    /// Device-reported read state.
    pub read: bool,
    /// Decoded message body text.
    pub text: String,
}
