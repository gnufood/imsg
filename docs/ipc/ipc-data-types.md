# IPC Data Types

The `imsg-ipc` crate defines the wire types exchanged between the CLI and the broker over a Unix domain socket. Serialisation uses `serde_json` with length-delimited framing. All types are public and versioned as part of the IPC contract.

## Message and Thread DTOs

### MessageDto

A single message row from a live `list` operation.

| Field | Type | Description |
|-------|------|-------------|
| `handle` | `String` | Opaque MAP message handle |
| `timestamp_ms` | `i64` | Message datetime in epoch milliseconds |
| `address` | `String` | Resolved peer address |
| `folder` | `String` | MAP folder name (`inbox`/`sent`/`outbox`/`deleted`) |
| `read` | `bool` | Device-reported read state |
| `text` | `String` | Decoded message body text |

### ThreadDto

A per-contact thread summary from a live `threads` operation. Counts are approximate — aggregated over the device's listing window, not the full corpus.

| Field | Type | Description |
|-------|------|-------------|
| `address` | `String` | Peer address grouping the thread |
| `latest_ms` | `i64` | Most recent message datetime in epoch milliseconds |
| `total` | `u32` | Total messages seen for this address across listed folders |
| `unread` | `u32` | Count of unread received messages |

### BodyDto

One message body from a live `get` operation. Carries no timestamp by design — a bMessage has no datetime.

| Field | Type | Description |
|-------|------|-------------|
| `handle` | `String` | Opaque MAP message handle the body was fetched by |
| `direction` | `Direction` | Direction derived from the bMessage folder |
| `address` | `String` | Resolved peer address |
| `folder` | `String` | MAP folder string from the bMessage (e.g. `telecom/msg/inbox`) |
| `read` | `bool` | Device-reported read state |
| `text` | `String` | Decoded message body text |

### Direction

Message direction relative to the paired device.

| Variant | Description |
|---------|-------------|
| `Received` | Inbound — received from the peer |
| `Sent` | Outbound — sent from this device |

## Contact and Phone DTOs

### ContactDto

One contact vCard from a live `get`/`lookup`/`pull_all` operation. Carries no handle by design — unlike `CardEntryDto`, a pulled vCard doesn't retain the (volatile, listing-scoped) handle it was fetched by.

| Field | Type | Description |
|-------|------|-------------|
| `display_name` | `Option<String>` | Value of the vCard FN property; `None` if absent |
| `uid` | `Option<String>` | Value of the vCard UID property; `None` if absent. The durable per-contact key. |
| `phones` | `Vec<PhoneDto>` | Non-empty TEL values in vCard order, each carrying raw and canonical forms |

### PhoneDto

One phone number in both its raw device-reported form and its canonical E.164 form.

| Field | Type | Description |
|-------|------|-------------|
| `raw` | `String` | The number exactly as the device's vCard reported it (whitespace-stripped) |
| `e164` | `Option<String>` | The canonical E.164 form, or `None` if the number could not be resolved |

`e164` is `None` when the number could not be normalised. The raw form is always carried so the client can honour `contacts --raw`, which must show the number as the device reported it.

### CardEntryDto

One phonebook listing entry from a live `list` operation.

| Field | Type | Description |
|-------|------|-------------|
| `handle` | `String` | Opaque PBAP vCard handle assigned by the remote, e.g. `"41.vcf"` |
| `name` | `Option<String>` | Display name from the listing XML `name` attribute; `None` if absent |

## Folder and Sync DTOs

### FolderDto

One MAP message folder from a live `folders` operation. The name is a single path segment relative to `telecom/msg` (e.g. `inbox`), not a full path — the device reports the level it was asked to list, never the parents above it.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Folder name decoded from the listing XML `name` attribute |

### SyncReportDto

Outcome of a contacts cache sync. Adjacently tagged for the same reason `BrokerResponse` is: the newtype variant wraps a struct, and internal tagging can't represent every shape this enum may grow.

| Variant | Description |
|---------|-------------|
| `UpToDate` | Device's phonebook watermark matched the cache; no vCards were fetched. Still a successful sync — the freshness markers are stamped either way. |
| `Refreshed(RefreshDto)` | A full refresh ran; see `RefreshDto`. |

### RefreshDto

Per-entry accounting for one phonebook refresh.

| Field | Type | Description |
|-------|------|-------------|
| `listed` | `usize` | Entries the device's listing reported, including the skipped owner card |
| `pull_failed` | `usize` | Entries whose vCard fetch failed; skipped, never fatal to the sync |
| `no_uid` | `usize` | Fetched vCards with no `UID`, which the cache can't key on |
| `written` | `usize` | Contacts written to the cache |
| `wiped` | `bool` | `true` when a changed device identity discarded the whole cache first; a first-ever sync leaves this `false` |

`written < listed` means the cache is an incomplete view of the device's phonebook. The counters deliberately don't sum to `listed`: the skipped `0.vcf` owner card has no counter.

## Session State

### SessionState

The broker's MAP session lifecycle state, as reported to the CLI over `BrokerResponse`. Describes the MAP/OBEX session only — never MNS/notification health.

| Variant | Description |
|---------|-------------|
| `Disconnected` | No connection attempted yet |
| `Connecting` | Establishing the RFCOMM/OBEX/MAP session (first attempt or a reconnect attempt) |
| `Active` | MAP session live; operations may run |
| `Reconnecting` | A live session dropped; backoff is running before the next attempt |
| `Failed` | Terminal: the connect budget was exhausted or a permanent error occurred. The broker exits. |

**State progression:** `Disconnected → Connecting → Active`, with `Active → Reconnecting → Active` on a recoverable drop and a terminal `Failed` when the connect budget is exhausted or a permanent error occurs.

**Method:** `is_connected() -> bool` — returns `true` only in `Active` state, i.e. when MAP operations can run now.

## Failure Reason

### Reason

An action-oriented failure reason returned in `BrokerResponse::Failed`. Each variant maps to a distinct user action; establishment stage (RFCOMM vs OBEX vs notification registration) is deliberately not encoded — it is diagnostic log detail, not a reason, because it does not change what the user should do.

| Variant | Description | User Action |
|---------|-------------|-------------|
| `NotReady` | The session was not `Active` before the request's deadline elapsed | Retry |
| `DeviceUnreachable` | The device could not be reached (link timeout/reset) | Bring the phone close, check Bluetooth |
| `ConnectionRefused` | The device refused the connection (auth/pairing/wrong channel) | Re-pair or fix config |
| `OperationFailed(String)` | A MAP operation was rejected by the device; the inner string is the device-reported detail | See detail |
| `Internal(String)` | An unexpected protocol/parse error; the inner string is diagnostic detail to report | Report the detail |

## Response Types

### BrokerResponse

A single response frame from the broker. For one-shot requests the broker sends exactly one frame. For `BrokerRequest::Watch` the broker sends a stream of `BrokerResponse::WatchEvent` frames until the client closes the connection.

Adjacently tagged (`kind`/`data`) so newtype variants carrying a `String` or `Reason` serialise — internal tagging cannot represent a newtype wrapping a non-map value.

| Variant | Description |
|---------|-------------|
| `Ok` | Operation completed successfully with no textual result |
| `Text(String)` | Operation completed; inner string is the human-readable outcome |
| `Error(String)` | IPC-plumbing failure (malformed frame, broker shutting down, serialisation). Distinct from a device/session failure, which is carried by `Failed`. |
| `Failed(Reason)` | A MAP operation or connection failed with a typed, action-oriented reason |
| `WatchEvent(WatchEvent)` | One MAP notification event; only appears in response to `BrokerRequest::Watch` |
| `Messages(Vec<MessageDto>)` | Live folder listing rows; sole answer to `BrokerRequest::ListMessages` |
| `Threads(Vec<ThreadDto>)` | Live per-contact thread summaries; sole answer to `BrokerRequest::Threads` |
| `Body(BodyDto)` | One live message body; sole answer to `BrokerRequest::GetMessage` |
| `Folders(Vec<FolderDto>)` | Live MAP folder rows in device-reported document order; sole answer to `BrokerRequest::Folders` |
| `StatusInfo { state, device, persistent }` | Broker health snapshot; always sent in response to `BrokerRequest::Status` |
| `ContactsSynced { report }` | Contacts cache sync completed; sole answer to `BrokerRequest::SyncContacts` |
| `ContactEntries(Vec<CardEntryDto>)` | Live phonebook listing rows; sole answer to `BrokerRequest::ListContacts` |
| `Contact(ContactDto)` | One live contact vCard fetched by handle; sole answer to `BrokerRequest::GetContact` |
| `ContactLookup(Option<ContactDto>)` | Live reverse-lookup result; `None` when the device reports no match. Sole answer to `BrokerRequest::LookupContact` |
| `Contacts(Vec<ContactDto>)` | Every live contact vCard in a phonebook; sole answer to `BrokerRequest::PullAllContacts` |

### WatchEvent

A MAP notification event flattened for cross-process transport. Mirrors `map_core::mns_event::MnsEvent`; fields absent in the `<event>` XML element are `None`.

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | `EventType` | Governs which of the fields below are populated |
| `handle` | `Option<String>` | Opaque MAP message handle; absent for memory-state events |
| `folder` | `Option<String>` | Current folder path; absent for memory-state events |
| `old_folder` | `Option<String>` | Previous folder path; present only for `MessageShift` events |
| `msg_type` | `Option<String>` | Message type string (e.g. `"SMS_GSM"`); absent for memory-state events |
| `datetime` | `Option<String>` | ISO 8601 basic datetime string (e.g. `"20260624T120000"`); present for `NewMessage` |

### EventType

MAP 1.4 MNS notification event type. Serialises as the bare variant-name string (e.g. `"NewMessage"`) — wire-compatible with any consumer that only ever read the raw string this field used to carry.

| Variant | Description |
|---------|-------------|
| `NewMessage` | A new message arrived in a folder |
| `DeliverySuccess` | An outgoing message was delivered to the peer |
| `SendingSuccess` | An outgoing message left the device successfully (delivery to the peer not confirmed) |
| `DeliveryFailure` | An outgoing message failed to be delivered to the peer |
| `SendingFailure` | An outgoing message failed to leave the device |
| `MessageDeleted` | A message was deleted from a folder |
| `MessageShift` | A message moved between folders |
| `MemoryFull` | Device message memory is full; new messages may not be received |
| `MemoryAvailable` | Device message memory freed up after a prior `MemoryFull` |
| `ReadStatusChanged` | A message's read/unread status changed on the device |

## Request Types

### BrokerRequest

A single request from a CLI process to the broker. Sent as one length-delimited JSON frame. The broker processes one request at a time; concurrent CLI invocations queue in the kernel accept backlog.

| Variant | Description |
|---------|-------------|
| `Sync { folder }` | Drain the outbox then backfill MAP messages. `folder` is MAP path (e.g. `"TELECOM/MSG/INBOX"`); `None` syncs all four standard folders. |
| `Send { number, message }` | Record and push an outgoing SMS to the device; broker handles the full send lifecycle. |
| `Delete { handle, folder }` | Set the deleted flag on the device and remove the message from the local store. |
| `Backfill` | Run an incremental catch-up sync across all folders. Runs `backfill_catch_up`; no folder scoping. |
| `ListMessages { folder, unread, from, since, limit, offset }` | Query the device live for a folder listing (non-opted-in `list`). No store write. |
| `GetMessage { handle }` | Fetch one message body live by handle (non-opted-in `get`). No store write. |
| `Threads` | Aggregate live Inbox+Sent listings into per-contact threads (non-opted-in `threads`). No store write. |
| `Folders` | List the device's MAP message folders under `telecom/msg` (`folders`). No store write. |
| `MarkReadDevice { handle }` | Mark a message read on the device only (non-opted-in `get --read`). No store write. |
| `SendLive { number, message }` | Push an outgoing SMS to the device only (non-opted-in `send`). No store write — fire-and-forget. |
| `ListContacts { path, limit, offset }` | Query the device live for a phonebook listing (`contacts --list`). No store write. |
| `GetContact { path, handle }` | Fetch one contact vCard live by handle (`contacts --get`). No store write. |
| `LookupContact { path, number }` | Reverse-look up a contact by phone number live (`contacts --lookup`), then pull its vCard. No store write. |
| `PullAllContacts { path, limit, offset }` | Pull every contact vCard in a phonebook live (`contacts` with no flags). No store write. |
| `Watch` | Stream MAP notification events; broker sends zero or more `WatchEvent` frames until the client closes the connection. |
| `SyncContacts` | Pull the main phonebook and upsert contact display names into the local contacts cache. |
| `Status` | Return the broker's current connection state; always answered with one frame. |
| `Shutdown` | Request a graceful stop. Answered with `Ok` once accepted, or `Error` if this broker instance has no shutdown coordinator (the ephemeral one-shot broker; only persistent/daemon mode supports this). |

## Protocol Constants

### MAX_FRAME_LEN

Maximum frame size for broker IPC frames, in bytes: 64 * 1024 (65536 bytes). Enforced identically by the broker (bind side) and the CLI (connect side). Large enough for any realistic SMS body; increase when MMS attachments are brokered.