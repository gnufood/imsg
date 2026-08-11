# IPC Protocol

The IPC protocol defines the wire contract between the CLI and the broker process. Communication occurs over a Unix domain socket using length-delimited JSON frames.

## Frame Format

Each frame consists of a 4-byte length prefix (big-endian u32) followed by a JSON payload. The maximum frame size is `MAX_FRAME_LEN` (64 KB).

```
[4-byte length][JSON payload]
```

Serialization uses `serde_json`. `BrokerRequest` uses an internally tagged enum (`tag = "op"`) with `rename_all = "snake_case"`. `BrokerResponse` uses an adjacently tagged enum (`tag = "kind"`, `content = "data"`) to ensure newtype variants serialize correctly.

## Maximum Frame Size

```rust
pub const MAX_FRAME_LEN: usize = 64 * 1024;
```

This constant is enforced identically by the broker (bind side) and the CLI (connect side). The limit accommodates any realistic SMS body. Increase this value when MMS attachments are brokered.

## BrokerRequest

A single request frame from the CLI to the broker. The broker processes one request at a time; concurrent CLI invocations queue in the kernel accept backlog.

| Variant | Description |
| ------- |-------------|
| `Sync { folder }` | Drain the outbox then backfill MAP messages. `folder` is a MAP path (e.g., `"TELECOM/MSG/INBOX"`); `None` syncs all four standard folders. |
| `Send { number, message }` | Record and push an outgoing SMS to the device; broker handles the full send lifecycle. |
| `Delete { handle, folder }` | Set the deleted flag on the device and remove the message from the local store. |
| `Backfill` | Run an incremental catch-up sync across all folders. |
| `ListMessages { folder, unread, from, since, limit, offset }` | Query the device live for a folder listing. No store write. |
| `GetMessage { handle }` | Fetch one message body live by handle. No store write. |
| `Threads` | Aggregate live Inbox+Sent listings into per-contact threads. No store write. |
| `Folders` | List the device's MAP message folders under `telecom/msg`. No store write. |
| `MarkReadDevice { handle }` | Mark a message read on the device only. No store write. |
| `SendLive { number, message }` | Push an outgoing SMS to the device only. No store write; fire-and-forget. |
| `ListContacts { path, limit, offset }` | Query the device live for a phonebook listing. No store write. |
| `GetContact { path, handle }` | Fetch one contact vCard live by handle. No store write. |
| `LookupContact { path, number }` | Reverse-look up a contact by phone number live, then pull its vCard. No store write. |
| `PullAllContacts { path, limit, offset }` | Pull every contact vCard in a phonebook live. No store write. |
| `Watch` | Stream MAP notification events. Broker sends zero or more `WatchEvent` frames until the client closes the connection. |
| `SyncContacts` | Pull the main phonebook and upsert contact display names into the local contacts cache. |
| `Status` | Return the broker's current connection state. Always answered with one frame. |
| `Shutdown` | Request a graceful stop. Answered with `Ok` once accepted, or `Error` if this broker instance has no shutdown coordinator. |

## BrokerResponse

A single response frame from the broker. For one-shot requests the broker sends exactly one frame. For `Watch` the broker sends a stream of `WatchEvent` frames until the client closes the connection.

| Variant | Description |
| ------- |-------------|
| `Ok` | Operation completed successfully with no textual result. |
| `Text(String)` | Operation completed; the inner string is the human-readable outcome. |
| `Error(String)` | IPC-plumbing failure (malformed frame, broker shutting down, serialisation). Distinct from a device/session failure, which is carried by `Failed`. |
| `Failed(Reason)` | A MAP operation or connection failed with a typed, action-oriented reason. |
| `WatchEvent(WatchEvent)` | One MAP notification event; only appears in response to `Watch`. |
| `Messages(Vec<MessageDto>)` | Live folder listing rows; sole answer to `ListMessages`. |
| `Threads(Vec<ThreadDto>)` | Live per-contact thread summaries; sole answer to `Threads`. |
| `Body(BodyDto)` | One live message body; sole answer to `GetMessage`. |
| `Folders(Vec<FolderDto>)` | Live MAP folder rows in device-reported document order; sole answer to `Folders`. |
| `StatusInfo { state, device, persistent }` | Broker health snapshot; always sent in response to `Status`. |
| `ContactsSynced { report }` | Contacts cache sync completed; sole answer to `SyncContacts`. |
| `ContactEntries(Vec<CardEntryDto>)` | Live phonebook listing rows; sole answer to `ListContacts`. |
| `Contact(ContactDto)` | One live contact vCard fetched by handle; sole answer to `GetContact`. |
| `ContactLookup(Option<ContactDto>)` | Live reverse-lookup result; `None` when the device reports no match. Sole answer to `LookupContact`. |
| `Contacts(Vec<ContactDto>)` | Every live contact vCard in a phonebook; sole answer to `PullAllContacts`. |

## Session State

The broker's MAP session lifecycle state, as reported to the CLI over `BrokerResponse`.

| State | Description |
|-------|-------------|
| `Disconnected` | No connection attempted yet. |
| `Connecting` | Establishing the RFCOMM/OBEX/MAP session (first attempt or a reconnect attempt). |
| `Active` | MAP session live; operations may run. |
| `Reconnecting` | A live session dropped; backoff is running before the next attempt. |
| `Failed` | Terminal: the connect budget was exhausted or a permanent error occurred. The broker exits. |

The progression is `Disconnected → Connecting → Active`, with `Active → Reconnecting → Active` on a recoverable drop and a terminal `Failed` when the connect budget is exhausted or a permanent error occurs.

`SessionState::is_connected()` returns `true` only in the `Active` state.

## Failure Reasons

An action-oriented failure reason returned in `BrokerResponse::Failed`.

| Reason | Description |
|--------|-------------|
| `NotReady` | The session was not `Active` before the request's deadline elapsed. Action: retry. |
| `DeviceUnreachable` | The device could not be reached (link timeout/reset). Action: bring the phone close, check Bluetooth. |
| `ConnectionRefused` | The device refused the connection (auth/pairing/wrong channel). Action: re-pair or fix config. |
| `OperationFailed(String)` | A MAP operation was rejected by the device; the inner string is the device-reported detail. |
| `Internal(String)` | An unexpected protocol/parse error; the inner string is diagnostic detail to report. |

Each variant maps to a distinct user action. Establishment stage (RFCOMM vs OBEX vs notification registration) is deliberately not encoded—it is diagnostic log detail, not a reason, because it does not change what the user should do.

## Event Types

MAP 1.4 MNS notification event types, carried in `WatchEvent`.

| EventType | Description |
|-----------|-------------|
| `NewMessage` | A new message arrived in a folder. |
| `DeliverySuccess` | An outgoing message was delivered to the peer. |
| `SendingSuccess` | An outgoing message left the device successfully (delivery to the peer not confirmed). |
| `DeliveryFailure` | An outgoing message failed to be delivered to the peer. |
| `SendingFailure` | An outgoing message failed to leave the device. |
| `MessageDeleted` | A message was deleted from a folder. |
| `MessageShift` | A message moved between folders. |
| `MemoryFull` | Device message memory is full; new messages may not be received. |
| `MemoryAvailable` | Device message memory freed up after a prior `MemoryFull`. |
| `ReadStatusChanged` | A message's read/unread status changed on the device. |

`EventType` serialises as the bare variant-name string (e.g., `"NewMessage"`), wire-compatible with any consumer that only ever read the raw string this field used to carry.

## Data Transfer Objects

The protocol uses the following DTOs for data transfer:

| DTO | Description |
|-----|-------------|
| `MessageDto` | One message row from a live listing: handle, timestamp_ms, address, folder, read, text |
| `ThreadDto` | One thread summary: address, latest_ms, total, unread |
| `BodyDto` | One message body: handle, direction, address, folder, read, text |
| `FolderDto` | One MAP message folder: name |
| `CardEntryDto` | One phonebook listing entry: handle, name |
| `ContactDto` | One contact vCard: display_name, uid, phones |
| `PhoneDto` | One phone number in raw and E.164 form |
| `SyncReportDto` | Outcome of a contacts cache sync: `UpToDate` or `Refreshed(RefreshDto)` |
| `RefreshDto` | Per-entry accounting for one phonebook refresh: listed, pull_failed, no_uid, written, wiped |
| `WatchEvent` | A MAP notification event flattened for cross-process transport |

`Direction` indicates message direction relative to the paired device: `Received` (inbound) or `Sent` (outbound).

## Protocol Constraints

- The broker processes one request at a time.
- Concurrent CLI invocations queue in the kernel accept backlog.
- For `Watch`, the broker sends a stream of `WatchEvent` frames until the client closes the connection.
- `Status` is always answered with exactly one frame.
- `Shutdown` is answered with `Ok` if accepted, or `Error` if the broker has no shutdown coordinator (ephemeral one-shot broker).
- Live-query operations (`ListMessages`, `GetMessage`, `Threads`, `Folders`, `ListContacts`, `GetContact`, `LookupContact`, `PullAllContacts`) perform no store write—they query the device directly.