# iroh API Reference

The iroh module provides a QUIC-based transport that replaces the TCP bridge for connecting to remote machines. It implements a hub-and-spoke architecture where the hub owns the RFCOMM link to a Bluetooth device and spokes connect to it over QUIC.

## Architecture Overview

The transport layer consists of three functional components:

| Component | Role | Location |
|-----------|------|----------|
| Hub | Accepts spoke connections, proxies MAP/PBAP to RFCOMM, streams MNS events | `hub.rs` |
| Spoke | Connects to hub, opens ALPN-tagged streams for MAP/PBAP/MNS | `spoke.rs` |
| Stream wrappers | Manages connection lifetime for OBEX session duration | `stream.rs` |

Each profile (MAP, PBAP, MNS) rides its own ALPN-tagged bidirectional QUIC stream. MAP and PBAP requests flow from spoke to hub and are proxied into RFCOMM. MNS events flow from hub to spoke.

## ALPN Protocols

The module defines three Application-Layer Protocol Negotiation identifiers:

```rust
pub const MAP_ALPN: &[u8] = b"imsg-map/1";
pub const PBAP_ALPN: &[u8] = b"imsg-pbap/1";
pub const MNS_ALPN: &[u8] = b"imsg-mns/1";
```

These ALPN values are advertised by both hub and spoke endpoints. The hub filters incoming connections by ALPN and routes them to the appropriate handler.

## Hub Runtime

The hub runs as an async task that accepts incoming QUIC connections and dispatches them based on ALPN.

### `run_hub`

```rust
pub async fn run_hub(
    key: SecretKey,
    bt_addr: Address,
    map_ch: u8,
    pbap_ch: u8,
    mns_events: broadcast::Sender<Bytes>,
    mut cancel: watch::Receiver<bool>,
) -> Result<(), TransportError>
```

Binds a QUIC endpoint with the `N0` preset (relay + discovery enabled) and accepts incoming spoke connections. For each connection:

- **MAP_ALPN**: Proxies the QUIC stream to RFCOMM channel `map_ch` for the Bluetooth device at `bt_addr`.
- **PBAP_ALPN**: Proxies the QUIC stream to RFCOMM channel `pbap_ch`.
- **MNS_ALPN**: Streams MNS event payloads to the connected spoke.

**Concurrency model**: At most one active proxy per channel (MAP or PBAP) at any time. Concurrent connections queue on a semaphore until the previous OBEX session completes. MNS connections have no such limit — all subscribed spokes receive every event.

**Shutdown behavior**: Returns once `cancel` receives `true` or the sender is dropped. Drawns in-flight proxy tasks before closing the endpoint.

**Errors**: Returns `TransportError::External` if the endpoint cannot bind.

### MNS Event Framing

MNS events are framed with a 4-byte big-endian length prefix followed by the raw event-report bytes:

```
[4-byte length (u32 BE)][payload bytes...]
```

Callers must read the 4-byte header and then exactly that many payload bytes per event.

## Spoke Endpoints

Spokes are client-side QUIC endpoints that connect to a hub. Each spoke reuses a single endpoint for all profile connections.

### `bind_spoke`

```rust
pub async fn bind_spoke() -> Result<Endpoint, TransportError>
```

Binds an ephemeral spoke endpoint with the `N0` preset. The returned endpoint is caller-owned and should be reused for all MAP/PBAP/MNS connections from that spoke.

**Errors**: Returns `TransportError::External` if no UDP socket can be bound.

### `connect_map_hub`

```rust
pub async fn connect_map_hub(
    endpoint: &Endpoint,
    hub: impl Into<EndpointAddr>,
) -> Result<ObexTransport<SpokeStream>, TransportError>
```

Opens an OBEX-framed MAP request stream to the hub over ALPN `MAP_ALPN`. The `hub` parameter accepts either an `EndpointId` (resolved via discovery/relay) or a full `EndpointAddr` with direct addresses.

**Does not perform an OBEX handshake** — the caller drives CONNECT on the returned transport.

**Errors**: Returns `TransportError::External` if the connection or stream cannot be established.

### `connect_pbap_hub`

```rust
pub async fn connect_pbap_hub(
    endpoint: &Endpoint,
    hub: impl Into<EndpointAddr>,
) -> Result<ObexTransport<SpokeStream>, TransportError>
```

Identical contract to `connect_map_hub` but for the PBAP profile over ALPN `PBAP_ALPN`.

### `connect_mns_hub`

```rust
pub async fn connect_mns_hub(
    endpoint: &Endpoint,
    hub: impl Into<EndpointAddr>,
) -> Result<HubRecvStream, TransportError>
```

Opens a persistent MNS subscription stream to the hub over ALPN `MNS_ALPN`. The spoke initiates the bidirectional stream and immediately drops its send half to signal that no spoke→hub data will arrive. The hub writes MAP event-report payloads to its send half.

Returns a `HubRecvStream` that bundles the receive half and a connection drop guard — the connection stays alive until the stream is dropped.

**Errors**: Returns `TransportError::External` if the connection cannot be established or the stream cannot be opened.

## Stream Wrappers

Two wrapper types manage QUIC connection lifetime, ensuring the underlying `Connection` stays alive for the duration of an OBEX session.

### `HubStream`

```rust
pub struct HubStream {
    inner: SpokeStream,
    conn: Connection,
}
```

A hub-side OBEX stream that keeps its `Connection` alive for the session duration. Wraps `SpokeStream` and owns the underlying `Connection` as a drop guard.

**Behavior**: Dropping `Connection` tears down all QUIC streams through it immediately. This wrapper prevents that until the OBEX session completes. On drop, closes the connection with error code 0 and an empty reason.

Implements `AsyncRead` and `AsyncWrite` by delegating to the inner `SpokeStream`.

### `HubRecvStream`

```rust
pub struct HubRecvStream {
    inner: RecvStream,
    conn: Connection,
}
```

An MNS subscription stream that keeps its `Connection` alive for the session duration. Wraps `RecvStream` and owns the underlying `Connection` as a drop guard.

**Behavior**: Same lifetime management as `HubStream`. Both `RecvStream` and `Connection` are `Unpin`, so this type is `Unpin` and can be passed directly to `tokio::io::AsyncReadExt` methods.

On drop, closes the connection with error code 0 and an empty reason.

## Key Management

### `load_or_create_key`

```rust
pub async fn load_or_create_key(path: &Path) -> Result<SecretKey, TransportError>
```

Loads a persisted hub secret key from `path`, or generates, persists, and returns a new one.

**Key file format**: Exactly 32 raw ed25519 seed bytes.

**File permissions**: On fresh key creation, parent directories are created if absent, the key is written with mode `0600`, and the in-memory key bytes are zeroized after writing.

**Errors**:
- `TransportError::Io` on directory creation, read, or write failure
- `TransportError::External` if an existing key file is not exactly 32 bytes

**Security note**: Does not verify that `path` is on an encrypted volume. Secure placement is the caller's responsibility.

## Type Aliases

```rust
pub type SpokeStream = Join<RecvStream, SendStream>;
```

A bidirectional QUIC stream presented to OBEX framing as a single duplex I/O object. Combines a `RecvStream` and `SendStream` using `tokio::io::join`.

## Re-exports

The module re-exports the following iroh types:

- `Endpoint` — QUIC endpoint for binding and connecting
- `EndpointAddr` — Endpoint address (ID or direct address)
- `EndpointId` — Relay-resolved endpoint identifier
- `SecretKey` — Ed25519 secret key for endpoint authentication
- `Connection` — Accepted or connected QUIC connection
- `RecvStream` — Incoming QUIC stream
- `SendStream` — Outgoing QUIC stream

## Error Handling

All iroh operations are wrapped with `iroh_err`, which converts any displayable error into `TransportError::External`:

```rust
pub(crate) fn iroh_err<E: std::fmt::Display>(e: E) -> TransportError {
    TransportError::External(e.to_string())
}
```

This ensures consistent error propagation through the OBEX transport layer.

## Relationship to RFCOMM

The iroh hub serves as a QUIC replacement for direct RFCOMM connections from the local machine. The hub owns the RFCOMM link and speaks QUIC to spokes. This enables:

- Remote operation where the Bluetooth device is attached to a different machine
- Multiple spokes sharing a single RFCOMM link (with serialization per profile)
- MNS event fan-out to multiple subscribed spokes

The RFCOMM connection logic itself resides in `crate::rfcomm`. The hub proxies QUIC streams to RFCOMM using `tokio::io::copy_bidirectional`.