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
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Inbound — received from the peer.
    Received,
    /// Outbound — sent from this device.
    Sent,
}

/// One message row from a live `list`, mirroring `session::live::models::LiveMessage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

/// Outcome of a contacts cache sync, mirroring `session::contacts::SyncReport`.
///
/// Adjacently tagged for the same reason [`crate::BrokerResponse`] is: the newtype variant
/// wraps a struct, and internal tagging can't represent every shape this enum may grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "outcome", content = "data", rename_all = "snake_case")]
pub enum SyncReportDto {
    /// Device's phonebook watermark matched the cache; no vCards were fetched. Still a
    /// successful sync — the freshness markers are stamped either way.
    UpToDate,
    /// A full refresh ran; see [`RefreshDto`].
    Refreshed(RefreshDto),
}

/// Per-entry accounting for one phonebook refresh, mirroring `session::contacts::Refresh`.
///
/// `written < listed` means the cache is an incomplete view of the device's phonebook. The
/// counters deliberately don't sum to `listed`: the skipped `0.vcf` owner card has no counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct RefreshDto {
    /// Entries the device's listing reported, including the skipped owner card.
    pub listed: usize,
    /// Entries whose vCard fetch failed; skipped, never fatal to the sync.
    pub pull_failed: usize,
    /// Fetched vCards with no `UID`, which the cache can't key on.
    pub no_uid: usize,
    /// Contacts written to the cache.
    pub written: usize,
    /// `true` when a changed device identity discarded the whole cache first; a first-ever sync
    /// leaves this `false`.
    pub wiped: bool,
}

/// One phonebook listing entry from a live `list`, mirroring `pbap_core::CardEntry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CardEntryDto {
    /// Opaque PBAP vCard handle assigned by the remote, e.g. `"41.vcf"`.
    pub handle: String,
    /// Display name from the listing XML `name` attribute; `None` if absent.
    pub name: Option<String>,
}

/// One phone number in both its raw device-reported form and its canonical E.164 form.
///
/// `e164` is `None` when the number could not be normalised. The raw form is always carried so
/// the client can honour `contacts --raw`, which must show the number as the device reported it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PhoneDto {
    /// The number exactly as the device's vCard reported it (whitespace-stripped).
    pub raw: String,
    /// The canonical E.164 form, or `None` if the number could not be resolved.
    pub e164: Option<String>,
}

/// One contact vCard from a live `get`/`lookup`/`pull_all`, mirroring
/// `formats::vcard::Contact`.
///
/// Carries no handle by design — unlike [`CardEntryDto`], a pulled vCard doesn't retain the
/// (volatile, listing-scoped) handle it was fetched by.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ContactDto {
    /// Value of the vCard FN property; `None` if absent.
    pub display_name: Option<String>,
    /// Value of the vCard UID property; `None` if absent. The durable per-contact key.
    pub uid: Option<String>,
    /// Non-empty TEL values in vCard order, each carrying raw and canonical forms.
    pub phones: Vec<PhoneDto>,
}
