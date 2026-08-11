# RFCOMM Transport Reference

The RFCOMM transport module provides Bluetooth Classic connectivity via the `bluer` crate, implementing RFCOMM channel establishment, link state monitoring, and MAP Message Notification Service (MNS) profile registration.

## Overview

The transport operates over Bluetooth Classic RFCOMM channels, which provide serial-port-like connections over Bluetooth. It is used by the MAP (Message Access Profile) and PBAP (Phone Book Access Profile) OBEX transports to communicate with paired Bluetooth devices.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `DEFAULT_BT_CONNECTED_GATE` | 2 seconds | Default timeout for the RFCOMM link to reach `BT_CONNECTED` state |
| `MNS_CHANNEL` | 17 | RFCOMM channel for MAP Message Notification Service |
| `MNS_UUID` | `00001133-0000-1000-8000-00805f9b34fb` | UUID for MAP Message Notification Service SDP record |

## Connection Establishment

### `connect`

```rust
pub async fn connect(
    addr: bluer::Address,
    channel: u8,
    bt_gate: Duration,
    security: Option<Security>,
) -> Result<Stream, TransportError>
```

Connects to a Bluetooth device at `addr` on RFCOMM `channel` without performing SDP service discovery.

**Behavior:**

1. Creates a new RFCOMM socket
2. If `security` is `Some`, requests the specified security level via `setsockopt(BT_SECURITY)` before connecting
3. Initiates the RFCOMM connection
4. Polls `peer_addr().is_ok()` every 25ms until it succeeds or `bt_gate` expires

**Rationale:** The `bluer` `Stream::connect` method returns while the RFCOMM channel is still in the `BT_CONNECT` state. The kernel confirms completion via `getpeername` succeeding, not via `SO_ERROR` (which returns 0 prematurely for RFCOMM). The polling loop ensures the first OBEX write never hits `ENOTCONN`.

**Errors:**

- `TransportError::Io` with `InvalidInput` kind if the `bt_gate` deadline overflows the monotonic clock
- `TransportError::Io` with `TimedOut` kind if the link does not reach `BT_CONNECTED` within `bt_gate`
- `TransportError::Io` for socket creation, security level rejection, or RFCOMM connect failure

**Security levels:** The `bluer::rfcomm::Security` enum specifies authentication, encryption, and bonding requirements. Passing `None` leaves the socket's security unmodified, using whatever the existing pairing/bond already negotiated.

## Profile Listener

### `ProfileListener`

```rust
pub struct ProfileListener {
    _session: bluer::Session,
    handle: std::pin::Pin<Box<bluer::rfcomm::ProfileHandle>>,
}
```

Yields incoming connection requests from remote devices. The `_session` field keeps the BlueZ session alive—dropping the `ProfileListener` unregisters the SDP profile from BlueZ.

**`next` method:**

```rust
pub async fn next(&mut self) -> Option<bluer::rfcomm::ConnectRequest>
```

Returns `None` when the profile is unregistered or BlueZ terminates the session.

## MNS Profile Registration

### `listen_mns`

```rust
pub async fn listen_mns() -> Result<ProfileListener, TransportError>
```

Registers the MAP Message Notification Service RFCOMM server profile with BlueZ on channel 17 and returns a `ProfileListener`.

**SDP Service Record:** The profile advertises the following SDP record:

- Service class ID: `0x1133` (MAP Message Notification Service)
- Protocol descriptor list: L2CAP → RFCOMM with channel 17
- Bluetooth profile descriptor list: MAP version 1.1
- Service name: "MAP Message Notification Service"
- Supported message types: 0x00 (SMS-GSM)
- Supported features: 0x02 (Notification)

**Errors:**

- `TransportError::Io` if the BlueZ D-Bus session cannot be established
- `TransportError::Io` if profile registration is rejected

**Constraints:** Does not validate that BlueZ is powered or that a device is paired. Caller must keep the returned `ProfileListener` alive for the duration of the server's operation.

## Link State Monitoring

### `link_events`

```rust
pub async fn link_events(
    addr: bluer::Address,
) -> Result<impl futures::Stream<Item = bool>, TransportError>
```

Subscribes to BlueZ's connection reports for a specific device address.

**Stream semantics:**

- Yields `true` when the device is reachable and connected
- Yields `false` when BlueZ reports the device as disconnected
- The first item is the state at subscription time, not a transition—a caller subscribing after the device already dropped receives `false` immediately
- The stream ends when BlueZ stops reporting entirely (bluetoothd restart, D-Bus loss, device removed)

**Important:** This reports the *device's* ACL link state, which is coarser than one profile's RFCOMM channel. A phone still connected for another profile reads as `true` even when the specific RFCOMM channel used by this transport is gone.

**Errors:**

- `TransportError::Io` if the D-Bus session or default adapter cannot be reached
- `TransportError::Io` if `addr` is unknown to BlueZ

## Internal: BT_CONNECTED Gate

### `await_bt_connected`

```rust
async fn await_bt_connected(
    is_connected: impl Fn() -> bool,
    deadline: tokio::time::Instant,
) -> Result<(), TransportError>
```

Polls the provided `is_connected` predicate every 25ms until it returns `true` or the deadline passes.

**Error:** Returns `TransportError::Io` with `TimedOut` kind if the deadline passes before `is_connected` succeeds.

**Test invariants (verified in `tests.rs`):**

- An unconnected socket's `peer_addr()` returns `Err` with `NotConnected` kind
- A connected socket's `peer_addr()` returns `Ok`

## Error Types

All errors are expressed as `TransportError`, defined in the `imsg-obex` crate and re-exported by `imsg-transport`:

| Variant | Description |
|---------|-------------|
| `TransportError::Io(std::io::Error)` | OS-level socket errors; inspect `kind()` for connection-refused, permission-denied, not-connected, timed-out, etc. |
| `TransportError::External(String)` | External transport driver errors (QUIC, TLS, or other); carries the driver error message |

## Usage with OBEX

The RFCOMM `Stream` returned by `connect` is a raw byte transport. To obtain an OBEX-framed transport, wrap it with `obex_core::wrap`:

```rust
use imsg_obex::ObexTransport;
use bluer::rfcomm::Stream;

fn wrap_rfcomm(stream: Stream) -> ObexTransport<Stream> {
    imsg_obex::wrap(stream)
}
```

The resulting `ObexTransport` yields and accepts complete OBEX packets as `bytes::Bytes`, suitable for use with `futures::SinkExt` and `futures::StreamExt`.

## Platform Requirements

- Linux with BlueZ (`bluetoothd`) running
- Bluetooth adapter powered on
- Target device paired with the host
- RFCOMM channel number resolved via SDP (see `discover::resolve_channels`)

## Related Modules

- [`discover`](discover.md): Paired device listing and SDP channel resolution
- [`iroh`](iroh.md): QUIC hub/spoke transport for remote machine connectivity
- [`tcp`](tcp.md): TCP stream connector for development proxy