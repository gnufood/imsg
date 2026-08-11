# Connection API

The Connection API establishes MAP (Message Access Profile) and PBAP (Phone Book Access Profile) sessions over either iroh hub (QUIC) or classic Bluetooth RFCOMM. It provides transport selection, OBEX handshake, error classification, and retry policy primitives.

## Transport Selection

The system supports two transport paths:

| Transport | Protocol | Use Case |
|-----------|----------|----------|
| iroh hub | QUIC over the iroh mesh | Remote spoke connections via the hub |
| RFCOMM | Classic Bluetooth RFCOMM | Direct local Bluetooth connections |

The `Target` enum in `conn.rs` resolves to one of these paths based on whether an iroh `Endpoint` is provided:

```rust
enum Target {
    Hub(EndpointId),      // iroh hub mode
    Rfcomm(Address, u8),  // RFCOMM with device address and channel
}
```

Transport selection is determined by whether `endpoint: Option<&Endpoint>` is `Some` (hub) or `None` (RFCOMM). The target resolution validates that hub mode has a valid `node_key` configured, and RFCOMM mode has a parseable Bluetooth address.

## Session Establishment

### MAP Session

`connect_map` establishes a MAP session with event notification support:

```rust
pub fn connect_map<'a>(
    cfg: &'a Config,
    endpoint: Option<&'a Endpoint>,
    device_override: Option<&'a str>,
) -> Pin<Box<dyn Future<Output = Result<MapClient<Stream>>> + Send + 'a>>
```

The function performs these steps:

1. **Target resolution**: Selects hub or RFCOMM based on endpoint presence
2. **Transport connection**: 
   - Hub: `ep.connect(id, MAP_ALPN).await` then `open_bi()`
   - RFCOMM: `transport::rfcomm::connect(addr, channel, ...)`
3. **OBEX handshake**: Calls `establish_map_session` which:
   - Sends OBEX CONNECT
   - Enables notification registration via `set_notification_registration(true)`

The returned `MapClient` must be held alive by the caller. iOS drops notification registration on OBEX DISCONNECT.

### PBAP Session

`connect_pbap` establishes a PBAP session:

```rust
pub fn connect_pbap<'a>(
    cfg: &'a Config,
    endpoint: Option<&'a Endpoint>,
    device_override: Option<&'a str>,
) -> Pin<Box<dyn Future<Output = Result<PbapClient<Stream>>> + Send + 'a>>
```

PBAP uses `PBAP_ALPN` for hub connections and direct RFCOMM for classic Bluetooth.

### Lifecycle Functions

`lifecycle.rs` provides lower-level session establishment:

```rust
pub async fn establish_map_session<T: AsyncRead + AsyncWrite + Unpin>(
    stream: T,
) -> Result<MapClient<T>, SessionError>

pub async fn connect_map(
    addr: Address,
    channel: u8,
    bt_gate: Duration,
    security: Option<bluer::rfcomm::Security>,
) -> Result<MapClient<bluer::rfcomm::Stream>, SessionError>

pub async fn connect_pbap(
    addr: Address,
    channel: u8,
) -> Result<PbapClient<bluer::rfcomm::Stream>, SessionError>
```

## Error Classification

`retry.rs` classifies connection failures to determine retry policy:

### Disposition Enum

```rust
pub enum Disposition {
    Transient,   // Recoverable — retry with backoff
    Permanent,   // Non-recoverable — fail fast
}
```

### Classification Rules

The `classify` function inspects the error chain:

| Error Type | Condition | Disposition |
|------------|-----------|-------------|
| `TransportError::Io` | `ConnectionRefused`, `PermissionDenied`, `InvalidInput` | Permanent |
| `TransportError::Io` | All other I/O errors (timeout, reset, broken pipe) | Transient |
| `MapError::Obex(ObexError::ConnectRejected(_))` | OBEX CONNECT rejected | Permanent |
| `MapError::ServerError(_)` | Server error response | Permanent |
| `MapError::InvalidInput(_)` | Invalid input | Permanent |
| `MapError` | All other variants | Transient |
| `SessionError::Pbap(_)` | All PBAP errors | Transient |

The classification defaults to `Transient` for unrecognized errors. This conservative approach bounds the cost of a wrong guess via the retry budget, whereas a wrong "permanent" verdict would abandon a recoverable link.

### Session Fatality

`outbox.rs` provides additional classification for live sessions:

```rust
pub const fn is_session_fatal(e: &MapError) -> bool

pub fn is_fatal_anyhow(e: &anyhow::Error) -> bool
```

A session is fatal when the transport stream has died:
- `MapError::Transport`
- `MapError::UnexpectedEof`

All other error variants leave the session alive and usable for subsequent operations.

## Retry Policy

The `backoff` function builds an exponential backoff schedule:

```rust
pub fn backoff(
    initial: Duration,
    max: Duration,
    max_attempts: u32,
) -> impl Iterator<Item = Duration> + Clone
```

**Behavior:**
- Delays double from `initial`, capped at `max`
- Returns exactly `max_attempts - 1` values (the gaps between attempts)
- Empty iterator when `max_attempts <= 1`
- No jitter — the kernel-atomic bind election guarantees one broker per device

**Example:** `backoff(500ms, 2s, 5)` yields `[500ms, 1s, 2s, 2s]`

## Session Errors

`SessionError` is the top-level error type:

```rust
pub enum SessionError {
    Map(#[from] MapError),      // OBEX MAP layer errors
    Pbap(#[from] PbapError),    // OBEX PBAP layer errors
    Transport(#[from] TransportError),  // RFCOMM or I/O errors
}
```

## Push Error Classification

`outbox.rs` classifies MAP push (send) errors:

```rust
pub const fn classify_push_error(e: &MapError) -> (OutboxStatus, OutgoingStatus)
```

| Error Type | OutboxStatus | OutgoingStatus |
|------------|--------------|----------------|
| `Transport` / `UnexpectedEof` | Unknown | Unknown |
| `InvalidInput` / `ServerError` | Failed | FailedPermanent |
| All others | Failed | FailedRetryable |

Transport errors map to `Unknown` because the PUT may have been transmitted before the connection dropped — reconciliation against the Sent folder is required.

## Hub ID Resolution

`resolve_hub_id` parses the configured hub node key:

```rust
pub fn resolve_hub_id(node_key: Option<&str>) -> Result<EndpointId>
```

**Errors:**
- `None` if `hub.node_key` is not set
- Parse error if the key is not a valid iroh public key

This function does not validate hub reachability or pairing — those errors surface at connect time.

## Stream Type

The unified stream type accommodates both transports:

```rust
pub type Stream = Either<HubStream, bluer::rfcomm::Stream>;
```

`Either` from `tokio_util::either` provides a unified interface for the two transport implementations.

## See Also

- [Live Query API](live-query-api.md) — reading messages directly from the device without store writes
- [Sync API](./sync-api.md) — store-integrated message and contact synchronization
- [Outbox API](./outbox-api.md) — outgoing message push and delivery tracking
- [MNS API](./mns-api.md) — Message Notification Service event handling