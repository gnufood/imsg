# Store API

The Store API provides an encrypted SQLCipher database for persisting messages, contacts, outbox entries, and sync cursors. All operations are asynchronous and serialized on a dedicated background thread via `tokio-rusqlite`.

## Opening the Database

```rust
pub async fn open(path: PathBuf, key: SecretBox<[u8; 32]>) -> Result<Store, Error>
```

Opens the SQLCipher database at `path`, unlocks it with the supplied 256-bit `key`, and runs any pending migrations before returning.

**Constraints:**
- `path` must point to a writable location; parent directories must exist.
- `key` is consumed and zeroed after the PRAGMA is issued.
- The key must be the first operation issued on the connection.

**Pragmas issued on open (in order):**
1. `key` — SQLCipher encryption key (hex-encoded)
2. `journal_mode = WAL` — Write-Ahead Logging for concurrency
3. `synchronous = NORMAL` — Balanced durability/performance
4. `foreign_keys = ON` — Referential integrity enforcement

**Errors:**
- `Error::Database` — File cannot be opened (bad path, corruption, or wrong key)
- `Error::Connection` — Async dispatch failure
- `Error::Migration` — Pending migration cannot be applied

## Error Types

| Variant | Cause |
|---------|-------|
| `Error::Database` | Rusqlite error opening the database file (bad path, wrong key, corruption) |
| `Error::Connection` | Tokio-rusqlite async task or channel failure |
| `Error::Migration` | Schema migration failed; binary may be a downgrade |
| `Error::InvalidTransition` | `resolve` called with `OutboxStatus::Queued`, which is not a valid transition target |

## Message Operations

### Insertion

```rust
pub async fn upsert(&self, msg: NewMessage) -> Result<(), Error>
```

Inserts a message; silently no-ops if `map_handle` already exists. The `address` field carries both raw and canonical (E.164) forms; both are stored.

```rust
pub async fn enqueue_send(
    &self,
    msg: NewMessage,
    command: &str,
    payload: &str,
    created_at: i64,
) -> Result<(i64, i64), Error>
```

Atomically inserts a speculative `messages` row and a linked `outbox` entry. The message is created with a placeholder `map_handle` of the form `"local:{outbox_id}"` which is later replaced by `promote_outgoing` once the device assigns a real handle. Returns `(message_rowid, outbox_id)`.

### Reading

```rust
pub async fn get_by_handle(&self, handle: &str) -> Result<Option<MessageRow>, Error>
```

Returns the message with the given `map_handle`, or `None` if absent.

```rust
pub async fn list_messages(
    &self,
    folder: Option<&str>,
    unread_only: bool,
    from: Option<&str>,
    since_ms: Option<i64>,
    limit: u16,
    offset: u16,
) -> Result<Vec<MessageRow>, Error>
```

Returns messages matching all supplied criteria, newest-first.

- `folder` — Restricts to a single folder leaf (`"inbox"`, `"sent"`, etc.); `None` searches all folders.
- `unread_only` — Adds `status = 0` filter.
- `from` — Matches `address` exactly (matched on canonical form when resolvable).
- `since_ms` — Inclusive lower bound on `timestamp_ms`.
- `limit` and `offset` — Pagination; pass `1024` and `0` for the default first page.

```rust
pub async fn threads(&self) -> Result<Vec<ThreadRow>, Error>
```

Returns a per-address thread summary, most-recent-first. Groups all stored messages by canonical address (E.164 when resolved, else raw), counting total messages and unread received messages. `contact_name` is joined via `contact_phones` on the canonical form, so a message and a contact phone that differ only in formatting still resolve.

```rust
pub async fn messages_since(&self, after_ms: i64) -> Result<Vec<MessageRow>, Error>
```

Returns all messages with `timestamp_ms` strictly greater than `after_ms`, ordered oldest-first. Pass `0` to retrieve all stored messages.

```rust
pub async fn max_timestamp(&self) -> Result<Option<i64>, Error>
```

Returns the maximum `timestamp_ms` across all stored messages, or `None` if the store is empty.

### Modification

```rust
pub async fn delete_by_handle(&self, handle: &str) -> Result<(), Error>
```

Deletes the message identified by `handle`. No-ops silently if absent.

```rust
pub async fn update_folder(&self, handle: &str, folder: &str) -> Result<(), Error>
```

Updates the `folder` column for the message. No-ops silently if absent.

```rust
pub async fn update_status(&self, handle: &str, status: i32) -> Result<(), Error>
```

Updates the `status` column for the message. No-ops silently if absent.

```rust
pub async fn update_outgoing_status(
    &self,
    handle: &str,
    status: OutgoingStatus,
) -> Result<(), Error>
```

Updates `outgoing_status` for the message. Used on push failure or ambiguous outcome. No-ops silently if absent.

```rust
pub async fn promote_outgoing(
    &self,
    old_handle: &str,
    new_handle: &str,
    status: OutgoingStatus,
) -> Result<(), Error>
```

Renames the placeholder `map_handle` to the device-assigned handle and simultaneously sets `outgoing_status`. Called on push success. No-ops silently if absent.

```rust
pub async fn reconcile_outgoing(&self, handle: &str) -> Result<(), Error>
```

Advances `outgoing_status` from `sent_unconfirmed` to `sent_confirmed` for `handle`. Called during Sent-folder backfill reconciliation. No-ops if the row does not exist or has a different status.

```rust
pub async fn complete_send(
    &self,
    outbox_id: i64,
    old_handle: &str,
    new_handle: &str,
    now_ms: i64,
) -> Result<(), Error>
```

Atomically resolves the outbox entry to `Sent` and promotes the speculative message handle. Both writes succeed or neither is visible.

## Contact Operations

### Insertion

```rust
pub async fn upsert_contacts(&self, contacts: Vec<NewContact>) -> Result<(), Error>
```

Upserts every contact in a single transaction. The `phones` field replaces the complete previously-cached set for each UID — PBAP has no per-number diff signal, so a number no longer present is dropped from the cache. If a phone number is currently owned by a different contact, the later entry steals ownership.

```rust
pub async fn clear_contacts(&self) -> Result<(), Error>
```

Deletes every cached contact and phone number. Used when the device reports a changed `DatabaseIdentifier`, meaning previously cached UIDs may no longer be valid.

### Reading

```rust
pub async fn get_contact(&self, uid: &str) -> Result<Option<ContactRow>, Error>
```

Returns the full contact for `uid`, including all cached phone numbers, or `None`.

```rust
pub async fn lookup_contact(&self, address: &str) -> Result<Option<ContactRow>, Error>
```

Returns the full contact that owns phone number `address`, matched on the canonical form (E.164 when resolved, else raw). Pass an already-canonical address so formatting differences don't cause a miss.

```rust
pub async fn list_contacts(
    &self,
    limit: u16,
    offset: u16,
) -> Result<Vec<ContactEntryRow>, Error>
```

Returns lightweight contact identities (UID + display name, no phone numbers), ordered by display name (case-insensitive) then UID.

```rust
pub async fn all_contacts(
    &self,
    limit: u16,
    offset: u16,
) -> Result<Vec<ContactRow>, Error>
```

Returns full contacts (with phone numbers), ordered by display name then UID.

### Metadata

```rust
pub async fn pbap_meta(&self) -> Result<PbapMeta, Error>
```

Returns the cached PBAP phonebook identity/version watermark; fields are `None` if never set.

```rust
pub async fn set_pbap_meta(&self, meta: &PbapMeta) -> Result<(), Error>
```

Persists the PBAP phonebook identity/version watermark. A `None` field leaves the previously cached value untouched.

```rust
pub async fn contacts_synced_at(&self) -> Result<Option<i64>, Error>
```

Returns the timestamp of the last successful contacts sync, or `None` if never synced.

```rust
pub async fn set_contacts_synced_at(&self, ms: i64) -> Result<(), Error>
```

Persists `ms` as the contacts-sync freshness anchor.

## Outbox Operations

```rust
pub async fn enqueue(
    &self,
    command: &str,
    payload: &str,
    local_message_id: Option<i64>,
    created_at: i64,
) -> Result<i64, Error>
```

Inserts a new outbox entry with `status = queued` and returns its auto-assigned `id`. `created_at` must be milliseconds since Unix epoch.

```rust
pub async fn resolve(
    &self,
    id: i64,
    status: OutboxStatus,
    now_ms: i64,
    error: Option<String>,
) -> Result<(), Error>
```

Advances the lifecycle state of outbox entry `id`. Terminal states (`sent`, `failed`, `unknown`) set `resolved_at = now_ms`. The non-terminal `sending` state sets `attempted_at = now_ms` instead. `error` is stored verbatim and should be `None` for successful transitions.

**Valid transition targets:** `Sending`, `Sent`, `Failed`, `Unknown`. `Queued` is not valid.

```rust
pub async fn pending(&self) -> Result<Vec<OutboxRow>, Error>
```

Returns all outbox entries with `status = queued`, ordered oldest first.

## Cursor Operations

```rust
pub async fn get_cursor(&self, folder: &str) -> Result<Option<FolderCursor>, Error>
```

Returns the sync cursor for `folder`, or `None` if the folder has never been synced.

```rust
pub async fn set_cursor(
    &self,
    folder: &str,
    last_sync_at: i64,
    highest_ts: i64,
    sync_status: FolderSyncStatus,
) -> Result<(), Error>
```

Upserts the sync cursor for `folder`; creates the row if absent, overwrites if present. Call with `sync_status = Complete` after a successful backfill and `Failed` on error.

```rust
pub async fn latest_sync_at(&self) -> Result<Option<i64>, Error>
```

Returns the most recent `last_sync_at` across all folder cursors, or `None` if no folder has completed a sync yet.

## Generic Meta Operations

```rust
pub async fn get_meta(&self, key: &str) -> Result<Option<String>, Error>
```

Returns the raw text value stored under `key` in the `meta` table, or `None` if absent.

```rust
pub async fn set_meta(&self, key: &str, value: &str) -> Result<(), Error>
```

Upserts `value` under `key` in the `meta` table, overwriting any existing entry.

## Data Types

### MessageRow

| Field | Type | Description |
|-------|------|-------------|
| `rowid` | `i64` | Auto-assigned store rowid; monotonically increasing |
| `map_handle` | `String` | MAP protocol message handle |
| `timestamp_ms` | `i64` | Milliseconds since Unix epoch; used for ordering |
| `folder` | `String` | MAP folder (e.g. `telecom/msg/inbox`) |
| `direction` | `Direction` | `Received` (0) or `Sent` (1) |
| `address` | `String` | Remote phone number or address |
| `status` | `i32` | Raw MAP message status integer |
| `synced_at` | `i64` | Milliseconds when written to the store |
| `text` | `String` | Decoded message body text |
| `outgoing_status` | `Option<OutgoingStatus>` | Outgoing delivery state; `None` for received messages |

### ThreadRow

| Field | Type | Description |
|-------|------|-------------|
| `address` | `String` | Contact address; always non-empty |
| `latest_ms` | `i64` | Most recent message timestamp |
| `total` | `i64` | Total message count across all folders |
| `unread` | `i64` | Count of unread received messages |
| `latest_outgoing_status` | `Option<OutgoingStatus>` | Delivery state of most recent message |
| `contact_name` | `Option<String>` | Cached PBAP display name, or `None` |

### ContactRow

| Field | Type | Description |
|-------|------|-------------|
| `uid` | `String` | vCard UID; the store's primary key |
| `display_name` | `Option<String>` | vCard FN property value |
| `phones` | `Vec<PhoneField>` | Cached phone numbers with raw and E.164 forms |

### OutboxRow

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Auto-assigned primary key |
| `command` | `String` | Verb identifying the operation (e.g. `"send_sms"`) |
| `payload` | `String` | Serialized parameters |
| `local_message_id` | `Option<i64>` | Rowid of linked speculative message |
| `status` | `OutboxStatus` | Current lifecycle state |
| `created_at` | `i64` | Creation timestamp (ms since epoch) |
| `attempted_at` | `Option<i64>` | Most recent push attempt timestamp |
| `resolved_at` | `Option<i64>` | Terminal state timestamp |
| `error` | `Option<String>` | Last failure description |

### FolderCursor

| Field | Type | Description |
|-------|------|-------------|
| `folder` | `String` | MAP folder leaf (e.g. `"inbox"`) |
| `last_sync_at` | `i64` | Last completed sync timestamp |
| `highest_ts` | `i64` | Highest timestamp seen during last sync |
| `sync_status` | `FolderSyncStatus` | Sync completion state |

### Direction

| Variant | Value | Description |
|---------|-------|-------------|
| `Received` | 0 | Inbound — received from remote device |
| `Sent` | 1 | Outbound — sent from this device |

### OutboxStatus

| Variant | Description |
|---------|-------------|
| `Queued` | Waiting for the sync worker to attempt the push |
| `Sending` | Push initiated; awaiting device acknowledgement |
| `Sent` | Device acknowledged the push successfully |
| `Failed` | Push failed with a definitive error; no auto-retry |
| `Unknown` | Connection dropped mid-push; outcome requires reconciliation |

### OutgoingStatus

| Variant | Description |
|---------|-------------|
| `Queued` | Outbox entry created; push not yet attempted |
| `Sending` | Push in progress |
| `SentUnconfirmed` | Device accepted the push; not yet confirmed by Sent folder |
| `SentConfirmed` | Confirmed present in device Sent folder via reconciliation |
| `FailedRetryable` | Push failed with a transient error; retry warranted |
| `FailedPermanent` | Push failed with a permanent error; no retry |
| `Unknown` | Connection dropped mid-push; outcome requires reconciliation |

### FolderSyncStatus

| Variant | Description |
|---------|-------------|
| `Never` | No sync has been attempted for this folder yet |
| `Syncing` | A sync is currently in progress |
| `Complete` | Last sync finished without error; `highest_ts` is reliable |
| `Failed` | Last sync ended with an error; `highest_ts` reflects last successful boundary |

### PbapMeta

| Field | Type | Description |
|-------|------|-------------|
| `database_id` | `Option<String>` | Hex-encoded `DatabaseIdentifier` |
| `primary_version` | `Option<String>` | Hex-encoded `PrimaryVersionCounter` |
| `secondary_version` | `Option<String>` | Hex-encoded `SecondaryVersionCounter` |

### Status Constants

| Constant | Value |
|----------|-------|
| `STATUS_UNREAD` | 0 |
| `STATUS_READ` | 1 |