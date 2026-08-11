# Data Types

This reference page describes the row structures, status enumerations, and constants used throughout the `imsg-store` crate to represent messages, outbox entries, sync cursors, and cached contacts.

## Message Status Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `STATUS_UNREAD` | `0` | Raw status column value for an unread message. |
| `STATUS_READ` | `1` | Raw status column value for a read message. |

These constants represent the raw integer values stored in the `status` column of the `messages` table. The interpretation is caller-defined; the store treats them as opaque integers.

## Message Direction

```rust
enum Direction {
    Received = 0,  // Inbound — received from the remote device
    Sent = 1,      // Outbound — sent from this device
}
```

The `Direction` enum indicates whether a message was received from the remote device or sent by the local device. It is persisted as `0` or `1` in the `direction` column and implements `ToSql` and `FromSql` for SQLite serialization.

## Message Row Types

### MessageRow

A message row as it exists in the store, including the auto-assigned rowid.

| Field | Type | Description |
|-------|------|-------------|
| `rowid` | `i64` | Auto-assigned store rowid; monotonically increasing within this database. |
| `map_handle` | `String` | MAP protocol message handle identifying the message within the remote folder. |
| `timestamp_ms` | `i64` | Milliseconds since Unix epoch; used for ordering and catch-up queries. |
| `folder` | `String` | MAP folder the message resides in (e.g., `telecom/msg/inbox`). |
| `direction` | `Direction` | Received vs. sent; persisted as 0/1. |
| `address` | `String` | Remote phone number or address associated with the message. |
| `status` | `i32` | Raw MAP message status integer; interpretation is caller-defined. |
| `synced_at` | `i64` | Milliseconds since Unix epoch when this message was written to the store. |
| `text` | `String` | Decoded message body text. |
| `outgoing_status` | `Option<OutgoingStatus>` | Outgoing delivery state; `None` for all received messages and for sent messages that pre-date the Phase 4 outbox. Non-`None` only on rows with `direction = Sent` created via `Store::enqueue_send`. |

### ThreadRow

A per-contact conversation thread summary returned by `Store::threads()`.

| Field | Type | Description |
|-------|------|-------------|
| `address` | `String` | Contact address; always non-empty (empty-address messages are excluded by the query). |
| `latest_ms` | `i64` | Milliseconds since Unix epoch of the most recent message in this thread. |
| `total` | `i64` | Total message count across all folders for this address. |
| `unread` | `i64` | Count of unread received messages (`status = 0`, `direction = Received`). |
| `latest_outgoing_status` | `Option<OutgoingStatus>` | Outgoing delivery state of the most recent message in this thread; `None` when the latest message is received or was synced before Phase 4. |
| `contact_name` | `Option<String>` | Cached PBAP display name for `address`, joined from `contacts` via `contact_phones`; `None` if no cached contact has this exact address (match is a raw string equality, not phone-normalized). |

### NewMessage

A message to be inserted; the store auto-assigns `rowid` on insert.

| Field | Type | Description |
|-------|------|-------------|
| `map_handle` | `String` | MAP protocol message handle identifying the message within the remote folder. |
| `timestamp_ms` | `i64` | Milliseconds since Unix epoch. |
| `folder` | `String` | MAP folder the message resides in (e.g., `telecom/msg/inbox`). |
| `direction` | `Direction` | Whether the message was received or sent. |
| `address` | `PhoneField` | Remote phone number, carrying the raw device-reported form and, when resolvable, its E.164 canonical form. Normalised at the session ingress boundary before construction. |
| `status` | `i32` | Raw MAP message status integer; interpretation is caller-defined. |
| `synced_at` | `i64` | Milliseconds since Unix epoch when this sync run fetched the message. |
| `text` | `String` | Decoded message body text. |
| `outgoing_status` | `Option<OutgoingStatus>` | Outgoing delivery state; `None` for received messages and for sync-ingested sent messages. Set to `Some(OutgoingStatus::Queued)` only for speculative rows created by `Store::enqueue_send`. |

## Outgoing Status Types

### OutboxStatus

Lifecycle state of a row in the `outbox` table.

```rust
enum OutboxStatus {
    Queued,      // Waiting for the sync worker to attempt the push.
    Sending,     // Push initiated; awaiting device acknowledgement.
    Sent,        // Device acknowledged the push successfully.
    Failed,      // Push failed with a definitive error; will not be retried automatically.
    Unknown,     // Connection dropped mid-push; outcome requires reconciliation to determine.
}
```

Progresses `queued` → `sending` → `sent` | `failed` | `unknown`. The `unknown` state is set when the connection drops after a push was initiated but before acknowledgement was received; reconciliation against the device Sent folder is required to resolve it.

### OutgoingStatus

Fine-grained state of an outgoing row in the `messages` table.

```rust
enum OutgoingStatus {
    Queued,           // Outbox entry created; push not yet attempted.
    Sending,          // Push in progress.
    SentUnconfirmed,  // Device accepted the push; not yet confirmed by the Sent folder.
    SentConfirmed,    // Confirmed present in the device Sent folder via reconciliation.
    FailedRetryable,  // Push failed with a transient error; a retry is warranted.
    FailedPermanent,  // Push failed with a permanent error; no retry will be attempted.
    Unknown,          // Connection dropped mid-push; outcome requires reconciliation to determine.
}
```

`NULL` for all received messages. Progresses from `queued` toward `sent_confirmed` or a terminal failure state. `unknown` requires reconciliation against the device Sent folder before the outcome can be recorded.

### OutboxRow

A row from the `outbox` table representing one pending or resolved outgoing intent.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Auto-assigned primary key. |
| `command` | `String` | Verb identifying the outgoing operation (e.g., `"send_sms"`). |
| `payload` | `String` | Serialised parameters for the command. |
| `local_message_id` | `Option<i64>` | Rowid of the speculative `messages` row created alongside this entry, if any. |
| `status` | `OutboxStatus` | Current lifecycle state of this outbox entry. |
| `created_at` | `i64` | Milliseconds since Unix epoch when this entry was created. |
| `attempted_at` | `Option<i64>` | Milliseconds since Unix epoch of the most recent push attempt, or `None` if not yet tried. |
| `resolved_at` | `Option<i64>` | Milliseconds since Unix epoch when the entry reached a terminal state, or `None` if active. |
| `error` | `Option<String>` | Last failure description when `status` is `failed` or `unknown`; `None` otherwise. |

## Cursor Types

### FolderSyncStatus

Completion state stored in `folder_cursors.sync_status`.

```rust
enum FolderSyncStatus {
    Never,    // No sync has been attempted for this folder yet.
    Syncing,  // A sync is currently in progress.
    Complete, // Last sync completed successfully.
    Failed,   // Last sync ended with an error; `highest_ts` reflects the last successful boundary.
}
```

`never` is the initial value before any sync attempt on a given folder. `complete` means the last sync finished without error and `highest_ts` is reliable.

### FolderCursor

Per-folder sync cursor stored in `folder_cursors`.

| Field | Type | Description |
|-------|------|-------------|
| `folder` | `String` | MAP folder leaf (e.g., `"inbox"`, `"sent"`). |
| `last_sync_at` | `i64` | Milliseconds since Unix epoch of the last completed sync run for this folder. |
| `highest_ts` | `i64` | Highest `timestamp_ms` seen during the last sync; used as the next pull boundary. |
| `sync_status` | `FolderSyncStatus` | Whether the last sync completed, is in progress, or failed. |

Replaces the single global `last_sync_at` anchor in the `meta` table. Each folder tracks its own progress so a partial sync on one folder never corrupts another.

## Contact Types

### ContactRow

A cached PBAP contact, keyed by its durable vCard `UID` — not the volatile PBAP list handle, which the device re-assigns on every listing.

| Field | Type | Description |
|-------|------|-------------|
| `uid` | `String` | vCard `UID` property value; the store's primary key for this contact. |
| `display_name` | `Option<String>` | vCard `FN` property value, or `None` if the device omitted it. |
| `phones` | `Vec<PhoneField>` | All cached phone numbers, each carrying the raw device-reported form and, when resolvable, its E.164 canonical form. |

### ContactEntryRow

A lightweight contact identity without phone numbers, returned by `Store::list_contacts()`.

| Field | Type | Description |
|-------|------|-------------|
| `uid` | `String` | vCard `UID` property value; the store's primary key for this contact. |
| `display_name` | `Option<String>` | vCard `FN` property value, or `None` if the device omitted it. |

### NewContact

One contact to upsert via `Store::upsert_contacts()`.

| Field | Type | Description |
|-------|------|-------------|
| `uid` | `String` | vCard `UID` property value; the store's primary key for this contact. |
| `display_name` | `Option<String>` | vCard `FN` property value, or `None` if the device omitted it. |
| `phones` | `Vec<PhoneField>` | Current phone numbers, each carrying the raw device-reported form and, when resolvable, its E.164 canonical form. |

`phones` replaces the complete previously-cached set for `uid` — PBAP has no per-number diff signal, so a number no longer present here is dropped from the cache.

### PbapMeta

Cached PBAP phonebook identity/version watermark, stored in the `meta` table.

| Field | Type | Description |
|-------|------|-------------|
| `database_id` | `Option<String>` | Hex-encoded `DatabaseIdentifier`; `None` if the device never reported one. |
| `primary_version` | `Option<String>` | Hex-encoded `PrimaryVersionCounter`; `None` if the device never reported one. |
| `secondary_version` | `Option<String>` | Hex-encoded `SecondaryVersionCounter`; `None` if the device never reported one. |

Lets `session::contacts::sync_contacts` skip a re-pull when the device reports unchanged values, and decide whether a changed `database_id` requires wiping the cache first.

## Address Normalization

Message addresses and contact phone numbers are stored with both raw and canonical (E.164) forms. The store uses `COALESCE(address_e164, address)` for queries to match across different formatting variants. For example, a message stored with address `"+44(0)1753866488"` will match a query for `"+441753866488"` because both normalize to the same E.164 form.

The `threads()` query joins contacts via `contact_phones` on the canonical form, so a message and a contact phone that differ only in formatting still resolve to the same contact name.