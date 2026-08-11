# Live Query API

The Live Query API provides direct, read-only access to messages and contacts on a connected device without persisting data to the local store or advancing cursors. It serves both the in-process CLI path and the broker dispatch path, returning lean models that contain only device-derived fields.

## Overview

The Live Query API consists of two parallel module groups:

| Module | Protocol | Description |
|--------|----------|-------------|
| `session::live` | MAP (Messaging Access Profile) | Message listing, thread aggregation, folder enumeration, body fetch |
| `session::contacts::live` | PBAP (Phone Book Access Profile) | Contact listing, vCard fetch, reverse lookup |

Both paths share a common design principle: they read directly from the device, normalize protocol types to lean models, and return those models without writing to the store. The protocol types (`BMessage`, `MessageEntry`, `Contact`) never escape the session boundary.

## Message Live Query (`session::live`)

### ListFilter

Filters for live message listing, mirroring `store::list_messages` semantics.

```rust
pub struct ListFilter {
    /// Keep only unread messages.
    pub unread: bool,
    /// Keep only messages whose resolved address equals this value exactly.
    pub from: Option<String>,
    /// Keep only messages at or after this epoch-millisecond datetime.
    pub since_ms: Option<i64>,
    /// Maximum rows after `offset`; `None` keeps the rest of the window.
    pub limit: Option<u16>,
    /// Rows to skip from the front of the newest-first window.
    pub offset: u16,
}
```

**Filtering behavior:**
- `unread` is pushed to the device via `FilterReadStatus`, so an unread-only request never fetches read message bodies.
- `from`, `since_ms`, `limit`, and `offset` are applied in memory over the device's listing window.
- The device returns its whole fixed window regardless of offset, so paging is meaningless against it.

### list

Lists a folder live and returns lean messages newest-first, after applying the filter.

```rust
pub async fn list<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    folder: Folder,
    filter: &ListFilter,
) -> anyhow::Result<Vec<LiveMessage>>
```

**Behavior:**
- Fetches the full device window (no `since_ms` passed to the device)
- Applies `unread` filter device-side via `ListMessagesFilter::read_status`
- Sorts results by `timestamp_ms` descending (newest-first)
- Applies remaining filters and windowing in memory

### threads

Aggregates Inbox and Sent listings into per-contact thread summaries, newest-first.

```rust
pub async fn threads<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
) -> anyhow::Result<Vec<LiveThread>>
```

**Behavior:**
- Lists both Inbox and Sent folders via `list_folder` (no body fetch)
- Groups messages by canonical peer address
- Computes `total` (all messages), `unread` (received and unread), and `latest_ms` (most recent datetime)
- Drops entries with empty addresses
- Counts are approximate: aggregated over the device's bounded listing window, not the full corpus

### folders

Lists the device's MAP message folders under `telecom/msg`, in device-reported document order.

```rust
pub async fn folders<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
) -> anyhow::Result<Vec<LiveFolder>>
```

**Behavior:**
- Navigates to `telecom/msg` first, ensuring the result is the message-folder level
- Leaves the client parked at that location
- Returns folder names as single path segments (e.g., `inbox`), not full paths

### get

Fetches one message body live by handle and normalizes it to a `LiveBody`.

```rust
pub async fn get<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    handle: String,
) -> anyhow::Result<LiveBody>
```

**Behavior:**
- Derives `direction` from the bMessage folder (contains `sent`/`outbox` → `Sent`, otherwise `Received`)
- Address is the originator for received messages and the first recipient for sent messages
- Carries no timestamp — a `BMessage` has no datetime field

## Contact Live Query (`session::contacts::live`)

### list

Lists contact identities in a phonebook path, windowed by limit/offset. No vCard bodies fetched.

```rust
pub async fn list<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    limit: Option<u16>,
    offset: u16,
) -> anyhow::Result<Vec<CardEntry>>
```

### get

Fetches one contact's full vCard by its current PBAP handle.

```rust
pub async fn get<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    handle: &str,
) -> anyhow::Result<Contact>
```

**Note:** The handle is volatile — it is re-resolved on every `list` call. Callers must not cache it across syncs. Use the contact's `UID` for durable identification.

### pull_all

Fetches every contact in a phonebook path, windowed by limit/offset.

```rust
pub async fn pull_all<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    limit: Option<u16>,
    offset: u16,
) -> anyhow::Result<Vec<Contact>>
```

### lookup

Reverse-looks-up a contact by phone number via device-side `SearchAttribute::Number`, then pulls its full vCard.

```rust
pub async fn lookup<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut PbapClient<T>,
    path: PhonebookPath,
    number: &str,
) -> anyhow::Result<Option<Contact>>
```

**Behavior:**
- Returns `None` if the device reports no match
- Matching is whatever the device's own search implements
- No client-side E.164 normalization is applied

## Data Models

### Direction

Message direction relative to the device.

```rust
pub enum Direction {
    Received,  // Inbound — received from the peer
    Sent,      // Outbound — sent from this device
}
```

### LiveMessage

One message from a live folder listing, body text fetched on demand.

```rust
pub struct LiveMessage {
    pub handle: String,        // Opaque MAP message handle
    pub timestamp_ms: i64,     // Message datetime in epoch milliseconds
    pub address: String,       // Resolved peer address (sender for received, recipient for sent)
    pub folder: String,        // MAP folder name (e.g., "inbox", "sent")
    pub read: bool,            // Device-reported read state
    pub text: String,          // Decoded message body text
}
```

### LiveThread

Per-contact thread summary aggregated from live folder listings.

```rust
pub struct LiveThread {
    pub address: String,  // Peer address grouping the thread; never empty
    pub latest_ms: i64,   // Most recent message datetime across listed folders
    pub total: u32,       // Total messages seen for this address
    pub unread: u32,      // Count of unread received messages (!sent && !read)
}
```

**Note:** Counts are approximate — aggregated over the device's bounded listing window, not the full corpus. No delivery badge is included; outgoing status is outbox-only with no device source.

### LiveFolder

One MAP message folder from a live folder listing.

```rust
pub struct LiveFolder {
    pub name: String,  // Folder name from the listing XML name attribute
}
```

### LiveBody

One message body fetched live by handle.

```rust
pub struct LiveBody {
    pub handle: String,    // Opaque MAP message handle
    pub direction: Direction,
    pub address: String,   // Resolved peer address
    pub folder: String,    // MAP folder string (e.g., "telecom/msg/inbox")
    pub read: bool,        // Device-reported read state
    pub text: String,      // Decoded message body text
}
```

**Note:** Carries no timestamp — a `BMessage` has no datetime field.

## Address Normalization

All live query functions normalize addresses using `PhoneField::display()` to produce a consistent E.164 format where possible. This ensures that formatting variants (e.g., `+1-555-123-4567` vs. `+15551234567`) collapse into a single thread or filter match.

## Comparison with Sync Path

| Aspect | Live Query | Sync Path |
|--------|------------|-----------|
| Store writes | None | Full persistence |
| Cursor advances | None | Updated after each folder |
| Data scope | Device window only | Full corpus |
| Models | Lean (no `rowid`, `synced_at`, `outgoing_status`) | Full store models |
| Use case | Real-time UI, immediate reads | Background persistence, full-text search |

The sync path (`sync::backfill_folder`, `contacts::sync::sync_contacts`) writes to the store and advances cursors for incremental updates. The live query path bypasses this entirely for scenarios requiring immediate, device-authoritative data without the latency of persistence.

## Errors

All live query functions return `anyhow::Result` with the following error categories:

- **Transport errors**: Connection failures, timeouts
- **Protocol errors**: MAP/PBAP operation failures (e.g., `SetFolder`, `ListMessages`, `GetMessage`, `PullVCard`)
- **Parse errors**: Malformed response XML

Individual contact pull failures in `pull_all` are logged and skipped rather than aborting the operation.