# Broker Client API Reference

The broker client library provides IPC primitives for communicating with the imsg broker process over a Unix domain socket. It is used by both the CLI and GUI binaries to send requests to and receive responses from a running broker or daemon.

## Overview

The client operates over an abstract Unix socket using length-delimited JSON frames. Each request receives exactly one response frame, except for the `Watch` request which streams events until the client disconnects.

The library is organized into these functional areas:

| Module | Purpose |
|--------|---------|
| `transport` | Frame-level socket I/O: connect, send, receive |
| `probe` | Reachability checks and retry-until-connectable |
| `query` | One-shot broker state queries |
| `read` | Live device reads returning typed rows |
| `write` | Device write operations (send, delete, sync) |
| `contacts` | Contacts cache synchronization |
| `response` | Response interpretation and error extraction |

## Transport

The transport layer handles the low-level socket connection and frame encoding. It does not auto-start the broker; callers must ensure it is already running.

### `send_request`

```rust
pub async fn send_request(addr: &str, req: BrokerRequest) -> Result<BrokerResponse>
```

Sends a request over a fresh connection and returns one response frame. The `addr` parameter is the Bluetooth MAC address (e.g., `"TE:ST:00:00:00:01"`) used to derive the abstract socket name.

**Errors:** Returns an error if the connection fails, the frame cannot be encoded, or the response cannot be decoded.

### Internal transport functions

These are documented for reference; they handle the frame-level protocol but are not part of the public API.

- `connect_raw(addr)` — establishes a `Framed` connection to the broker's abstract socket using `LengthDelimitedCodec`
- `send_frame(framed, req)` — encodes `BrokerRequest` as JSON and writes one length-delimited frame
- `recv_frame(framed)` — reads one response frame and deserializes it to `BrokerResponse`

### Frame format

- Maximum frame size: 64 KB (`MAX_FRAME_LEN`)
- Encoding: JSON
- Framing: `tokio-util` `LengthDelimitedCodec` (4-byte length prefix)

## Reachability

### `probe`

```rust
pub async fn probe(addr: &str) -> bool
```

Returns `true` if the abstract broker socket for `addr` is currently connectable. Does not send any request; a successful connect probe indicates a broker is running.

### `connect_retry`

```rust
pub async fn connect_retry(
    addr: &str,
    child: &mut Child,
    log_path: &Path,
    deadline_in: Duration,
    poll: Duration,
) -> Result<()>
```

Retries connecting to the broker socket until success or failure. Probes every `poll` interval. Succeeds when the connect returns `Ok`; fails immediately when `child` exits before that, or after `deadline_in` elapses.

**Errors:** Returns an error if the child process exits before the socket becomes connectable, or if the deadline elapses.

## Queries

### `query_persistent`

```rust
pub async fn query_persistent(addr: &str) -> Option<bool>
```

Returns whether a reachable broker is running in persistent (daemon) mode, or `None` if no broker answers at `addr`.

Distinguishes an already-running daemon from an ephemeral one-shot broker that happens to be holding the socket—both answer a raw connect probe identically, so only the `Status` response's `persistent` field tells them apart.

### `query_state`

```rust
pub async fn query_state(addr: &str) -> Option<SessionState>
```

Returns the current session state of a reachable broker, or `None` if nothing answers at `addr` yet.

## Read Operations

### `folders`

```rust
pub async fn folders(addr: &str) -> Result<Vec<FolderDto>, ReadError>
```

Lists the device's MAP message folders through the broker, in device-reported document order. Goes through the broker rather than opening a MAP connection directly, sharing the one RFCOMM channel the broker already holds.

**Errors:** Returns `ReadError::Connect` if the broker cannot be reached, or `ReadError::Call` if it rejects the request.

### `ReadError`

```rust
pub enum ReadError {
    Connect(#[from] anyhow::Error),
    Call(#[from] CallError),
}
```

- `Connect` — the broker could not be reached, or the request/response frame was malformed
- `Call` — the broker was reachable but rejected the request or answered unexpectedly

## Write Operations

### `send`

```rust
pub async fn send(addr: &str, number: String, message: String) -> Result<String, WriteError>
```

Records and pushes an outgoing SMS; the broker enqueues it in its own store's outbox and tracks delivery.

**Errors:** Returns `WriteError::Connect` if the broker cannot be reached, or `WriteError::Call` if it rejects the request.

### `delete`

```rust
pub async fn delete(addr: &str, handle: String, folder: String) -> Result<String, WriteError>
```

Sets the deleted flag on the device and removes the message from the broker's store.

**Errors:** Returns `WriteError::Connect` if the broker cannot be reached, or `WriteError::Call` if it rejects the request.

### `sync`

```rust
pub async fn sync(addr: &str, folder: Option<String>) -> Result<String, WriteError>
```

Backfills MAP folders since their per-folder cursor anchors. The `folder` parameter limits the backfill to one MAP path; `None` syncs all four standard folders.

**Errors:** Returns `WriteError::Connect` if the broker cannot be reached, or `WriteError::Call` if it rejects the request.

### `WriteError`

```rust
pub enum WriteError {
    Connect(#[from] anyhow::Error),
    Call(#[from] CallError),
}
```

- `Connect` — the broker could not be reached, or the request/response frame was malformed
- `Call` — the broker was reachable but rejected the request or answered unexpectedly

## Contacts

### `sync_contacts`

```rust
pub async fn sync_contacts(addr: &str) -> Result<SyncReportDto, ContactsError>
```

Pulls the main phonebook and upserts contact display names into the broker's local cache, returning a `SyncReportDto` describing what the sync did.

**Errors:** Returns `ContactsError::Connect` if the broker cannot be reached, or `ContactsError::Call` if it rejects the request.

### `ContactsError`

```rust
pub enum ContactsError {
    Connect(#[from] anyhow::Error),
    Call(#[from] CallError),
}
```

- `Connect` — the broker could not be reached, or the request/response frame was malformed
- `Call` — the broker was reachable but rejected the request or answered unexpectedly

## Response Interpretation

### `CallError`

```rust
pub enum CallError {
    Failed(Reason),
    Error(String),
    Unexpected(Box<BrokerResponse>),
}
```

- `Failed` — the device/session rejected the operation
- `Error` — IPC-plumbing failure reported by the broker (malformed frame, broker shutting down)
- `Unexpected` — a response shape the caller did not expect (e.g., `Messages`, `WatchEvent`)

### Result extractors

- `text_result(resp)` — extracts `BrokerResponse::Text` or a `CallError`
- `folders_result(resp)` — extracts `BrokerResponse::Folders` or a `CallError`. An empty listing is a success, not an error.
- `contacts_synced_result(resp)` — extracts `BrokerResponse::ContactsSynced` or a `CallError`

## IPC Protocol Types

The client uses the `ipc` crate's protocol types:

### `BrokerRequest`

A single request from a client to the broker. Variants include:

| Variant | Description |
|---------|-------------|
| `Sync { folder }` | Drain outbox and backfill MAP messages |
| `Send { number, message }` | Record and push outgoing SMS |
| `Delete { handle, folder }` | Mark message deleted on device |
| `Folders` | List MAP message folders |
| `Status` | Query broker state |
| `Shutdown` | Request graceful stop (daemon only) |
| `Watch` | Stream MAP notification events |
| `SyncContacts` | Pull phonebook into local cache |

### `BrokerResponse`

A single response frame from the broker:

| Variant | Description |
|---------|-------------|
| `Ok` | Operation completed with no result |
| `Text(String)` | Operation completed with text result |
| `Error(String)` | IPC-plumbing failure |
| `Failed(Reason)` | Device/session operation failure |
| `StatusInfo { state, device, persistent }` | Broker health snapshot |
| `Folders(Vec<FolderDto>)` | MAP folder listing |
| `ContactsSynced { report }` | Contacts sync result |

### `SessionState`

The broker's MAP session lifecycle state:

| State | Description |
|-------|-------------|
| `Disconnected` | No connection attempted yet |
| `Connecting` | Establishing RFCOMM/OBEX/MAP session |
| `Active` | MAP session live; operations may run |
| `Reconnecting` | Session dropped; backoff running |
| `Failed` | Terminal; connect budget exhausted |

### `Reason`

An action-oriented failure reason:

| Reason | Description | User Action |
|--------|-------------|-------------|
| `NotReady` | Session not active before deadline | Retry |
| `DeviceUnreachable` | Link timeout/reset | Bring phone close, check Bluetooth |
| `ConnectionRefused` | Auth/pairing/wrong channel | Re-pair or fix config |
| `OperationFailed(detail)` | Device rejected operation | See detail |
| `Internal(detail)` | Protocol/parse error | Report detail |

## Socket Naming

The broker socket is an abstract Unix socket. The name is derived from the device address via `config::broker_abstract_name(addr)`, which produces a namespaced name like `@imsg::TE:ST:00:00:00:01`. The leading `@` indicates an abstract socket (no filesystem path required).

## Error Handling Patterns

The client uses a two-tier error model:

1. **Transport errors** (`anyhow::Error`) — connection failures, frame encoding/decoding failures
2. **Application errors** (`CallError`) — broker rejected the request or returned an unexpected response type

High-level operations (`send`, `delete`, `sync`, `folders`, `sync_contacts`) surface both tiers through their error types, allowing callers to distinguish between "broker unreachable" and "broker rejected the operation."

## See Also

- [Broker Protocol Reference](./broker_protocol.md) — full request/response specification
- [Broker Configuration](./broker_config.md) — startup timing and lifecycle policy
- [Session State Machine](./session_state.md) — MAP connection lifecycle details