# Broker Client Workflows

This document traces three end-to-end scenarios that illustrate how the broker client communicates with the imsg broker daemon over Unix abstract sockets. The client library (`imsg-broker-client`) provides reusable primitives for any consumer — the CLI or the GUI — to interact with a running broker without duplicating transport or protocol logic.

## Operation Overview

| Scenario | Starting Condition | Outcome |
|----------|-------------------|---------|
| **Probe broker availability** | Client holds a broker address string | Returns `true` if the abstract socket is connectable, `false` otherwise |
| **Send an SMS** | Client holds a phone number and message text | Returns a confirmation string from the broker, or propagates a failure |
| **Query session state** | Client holds a broker address string | Returns the current `SessionState` (e.g., `Active`, `Reconnecting`) or `None` if unreachable |

All three flows share the same transport foundation: a length-delimited JSON frame exchange over a Unix abstract socket. The transport layer is the common component that enables each operation to reach the broker.

## Shared Transport Foundation

Before examining individual operations, it is useful to understand the transport layer that every operation builds upon. The `transport` module provides three core functions:

- **`connect_raw(addr)`** — Establishes a fresh connection to the broker's abstract socket. The address is converted to an abstract socket name via `config::broker_abstract_name`. Returns a `Framed` stream using `LengthDelimitedCodec` with a maximum frame length defined by `MAX_FRAME_LEN` from the `ipc` crate.

- **`send_frame(framed, req)`** — Serializes a `BrokerRequest` to JSON, wraps it in a length-delimited frame, and sends it over the socket.

- **`recv_frame(framed)`** — Reads one response frame, deserializes it to a `BrokerResponse`, and returns it.

The public entry point `send_request(addr, req)` composes these three steps: connect, send the request, receive the response. This function is the workhorse used by higher-level operations in the `write`, `query`, `read`, and `contacts` modules.

The transport does not auto-start the broker — callers must ensure the broker is already running. This is a deliberate boundary: the client library focuses on communication, while process spawning and lifecycle management belong to `imsg-proc`.

---

## Scenario 1: Probe Broker Availability

### Purpose

Before attempting any operation, a client may need to verify that the broker is reachable. This is useful for UI decisions (showing a "disconnected" indicator) and for determining whether to spawn a new broker or connect to an existing one.

### Participating Components

- **`probe::probe(addr)`** — The public API for availability checks.
- **`transport::connect_raw(addr)`** — Used internally to attempt a socket connection.
- **`interprocess::local_socket::ConnectOptions`** — The underlying socket connection API.

### Flow

1. **Client calls `probe(addr)`** with a broker address string (e.g., `"TE:ST:00:00:00:01"`).

2. **`probe` resolves the abstract socket name** by calling `config::broker_abstract_name(addr)`. If this fails, it returns `false` immediately — the address itself is malformed.

3. **`probe` attempts a connection** using `ConnectOptions::new().name(name).connect_tokio()`. This is a non-blocking async connect that attempts to open the Unix abstract socket.

4. **If the connect succeeds**, the socket is immediately dropped (the client only cares that the socket exists, not that it maintains a connection), and `probe` returns `true`.

5. **If the connect fails**, `probe` returns `false`.

### State Transitions

- No persistent state is modified. The probe is a read-only reachability check.
- The broker itself is unaware that a probe occurred — the connect attempt may appear in its socket backlog, but no request is sent.

### Failure Conditions

- **Malformed address**: `config::broker_abstract_name` returns an error, and `probe` returns `false`.
- **Socket not listening**: The underlying connect fails, and `probe` returns `false`.

### Retry Variant

For scenarios where the broker is expected to start shortly (e.g., after spawning a new broker process), the `probe::connect_retry` function provides a retry loop:

```rust
pub async fn connect_retry(
    addr: &str,
    child: &mut Child,
    log_path: &Path,
    deadline_in: Duration,
    poll: Duration,
) -> Result<()>
```

This function polls `probe(addr)` at intervals until either the socket becomes connectable, the child process exits, or the deadline elapses. It is used during broker startup to wait for the spawned process to bind its socket before proceeding with operations.

---

## Scenario 2: Send an SMS

### Purpose

The primary write operation: sending an outgoing SMS through the broker, which enqueues it in the device's MAP outbox and tracks delivery status.

### Participating Components

- **`write::send(addr, number, message)`** — The public API for SMS sending.
- **`transport::send_request(addr, req)`** — Sends the request and receives the response.
- **`ipc::BrokerRequest::Send`** — The request variant containing the phone number and message.
- **`ipc::BrokerResponse`** — The response type, expected to be `BrokerResponse::Text` on success.
- **`response::text_result(resp)`** — Interprets the response, extracting the success string or mapping failures to `CallError`.

### Flow

1. **Client calls `send(addr, number, message)`** with the broker address, phone number, and message text.

2. **`send` constructs a `BrokerRequest::Send` variant**:
   ```rust
   BrokerRequest::Send { number, message }
   ```

3. **`send` calls `send_request(addr, req)`** from the transport module. This performs:
   - **Connect**: `connect_raw(addr)` establishes a framed connection to the abstract socket.
   - **Send**: `send_frame` serializes the `BrokerRequest::Send` to JSON and writes a length-delimited frame.
   - **Receive**: `recv_frame` reads one response frame and deserializes it to a `BrokerResponse`.

4. **`send` passes the response to `text_result(resp)`** from the response module. This function:
   - Returns `Ok(String)` if the response is `BrokerResponse::Text(s)` — the broker's confirmation message.
   - Returns `Err(CallError::Failed(reason))` if the response is `BrokerResponse::Failed` — the device or session rejected the operation.
   - Returns `Err(CallError::Error(msg))` if the response is `BrokerResponse::Error` — an IPC-level failure in the broker.
   - Returns `Err(CallError::Unexpected(...))` for any other response shape, which indicates a protocol mismatch.

5. **`send` returns the result**, propagating any error as a `WriteError`.

### State Transitions

- **Client side**: No persistent state change. The client receives a confirmation string or an error.
- **Broker side**: The broker receives the `Send` request, enqueues the message in its outbox, and initiates MAP `SetMessage` on the device. The broker's internal state (outbox queue, delivery tracking) is updated.
- **Device side**: The message is sent over the Bluetooth MAP RFCOMM channel managed by the broker.

### Failure Conditions

- **Broker unreachable**: `send_request` fails with an `anyhow::Error`, which `send` wraps as `WriteError::Connect`.
- **Device rejected the message**: The broker responds with `BrokerResponse::Failed(Reason)`, which `text_result` converts to `WriteError::Call(CallError::Failed(...))`.
- **Broker internal error**: The broker responds with `BrokerResponse::Error(msg)`, which `text_result` converts to `WriteError::Call(CallError::Error(...))`.
- **Protocol mismatch**: The broker responds with an unexpected variant (e.g., `Folders`), which `text_result` converts to `WriteError::Call(CallError::Unexpected(...))`.

### Related Write Operations

The `write` module also provides:
- **`delete(addr, handle, folder)`** — Marks a message as deleted on the device and removes it from the broker's store.
- **`sync(addr, folder)`** — Backfills MAP folders from the device, optionally limited to a specific folder.

All three follow the same pattern: construct a `BrokerRequest` variant, send it via `send_request`, and interpret the response via `text_result`.

---

## Scenario 3: Query Session State

### Purpose

Retrieve the current state of the broker's Bluetooth session — whether it is active, reconnecting, or disconnected — without performing a write operation. This is used for status displays and to distinguish between a persistent daemon and an ephemeral one-shot broker.

### Participating Components

- **`query::query_state(addr)`** — Returns the current `SessionState` or `None`.
- **`query::query_persistent(addr)`** — Returns `Some(true)` for a daemon, `Some(false)` for an ephemeral broker, or `None` if unreachable.
- **`transport::{connect_raw, send_frame, recv_frame}`** — Used directly (not via `send_request`) because the query needs to keep the connection open while interpreting the response.
- **`ipc::BrokerRequest::Status`** — The request variant that asks the broker for its status.
- **`ipc::BrokerResponse::StatusInfo`** — The response variant containing `state`, `device`, and `persistent` fields.

### Flow

1. **Client calls `query_state(addr)`** (or `query_persistent`).

2. **`query_state` establishes a connection** by calling `connect_raw(addr)`. If this fails, it returns `None` immediately.

3. **`query_state` sends a `BrokerRequest::Status`** by calling `send_frame`. If this fails, it returns `None`.

4. **`query_state` receives the response** by calling `recv_frame`. If this fails, it returns `None`.

5. **`query_state` matches on the response**:
   - If the response is `BrokerResponse::StatusInfo { state, .. }`, it extracts `state` and returns `Some(state)`.
   - For any other response variant, it returns `None`.

6. **`query_persistent` follows the same steps**, but extracts the `persistent` field from `StatusInfo` to distinguish a daemon from an ephemeral broker.

### State Transitions

- **Client side**: No persistent state change. The query is read-only.
- **Broker side**: The broker processes the `Status` request and returns its current session state. The broker's internal state is not modified.

### Failure Conditions

- **Broker unreachable**: `connect_raw` fails, and the function returns `None`.
- **Send or receive fails**: The function returns `None`.
- **Unexpected response shape**: The function returns `None` (this is a protocol mismatch, not a typical failure).

### Distinguishing Daemon from Ephemeral Broker

Both a persistent daemon and an ephemeral one-shot broker answer connection probes identically — the socket is bound and accept() works. Only the `Status` response's `persistent` field reveals the difference. This is why `query_persistent` exists: it allows callers to decide whether to spawn a new broker or reuse an existing one.

---

## Response Interpretation

The `response` module centralizes the interpretation of `BrokerResponse` variants. Rather than having each operation match the response inline, the module provides typed extractors:

- **`text_result(resp)`** — For operations that return a text confirmation (`send`, `delete`, `sync`).
- **`folders_result(resp)`** — For operations that return folder listings (`read::folders`).
- **`contacts_synced_result(resp)`** — For operations that return a sync report (`contacts::sync_contacts`).

Each extractor follows the same pattern: return the success payload for the expected variant, or a `CallError` for any failure variant (`Failed`, `Error`, or unexpected). This ensures consistent error handling across all broker operations.

---

## Summary

The broker client library provides a layered architecture for communicating with the imsg broker:

1. **Transport layer** (`transport`) handles the low-level socket connection, JSON serialization, and length-delimited frame exchange. It is the foundation that all operations share.

2. **Operation modules** (`probe`, `write`, `query`, `read`, `contacts`) build on the transport layer to provide typed APIs for specific broker operations. Each module constructs the appropriate `BrokerRequest` variant and interprets the response using the response module.

3. **Response module** (`response`) centralizes response interpretation, converting `BrokerResponse` variants into success payloads or typed `CallError` values.

The three scenarios above illustrate how these layers interact:
- **Probe** uses the transport's raw connect capability to check reachability.
- **Send SMS** uses the full request-response cycle via `send_request` and interprets the text response.
- **Query state** uses the transport directly to send a `Status` request and extract structured state information.

All operations are asynchronous (using Tokio) and follow the same error-handling pattern: transport-level failures become `Connect` errors, while broker-level rejections become `Call` errors. This allows callers to distinguish between "the broker is down" and "the device rejected the operation" and respond accordingly.