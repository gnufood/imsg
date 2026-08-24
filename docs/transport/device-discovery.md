# Device Discovery Reference

Device discovery provides paired Bluetooth device listing and SDP-based RFCOMM channel resolution for the MAP and PBAP profiles. It is used by both the CLI's `config setup` subcommand and the GUI's device configuration screen.

## Overview

The discovery module operates in two phases:

1. **Paired device listing** — Enumerate devices paired with the system via the default BlueZ adapter.
2. **SDP channel resolution** — Query each device's SDP server for the RFCOMM channel numbers used by MAP and PBAP service records.

These phases are independent. A caller may list devices without resolving channels, resolve channels for a specific device without listing, or combine them in a workflow.

## Paired Device Listing

### `PairedDevice`

A Bluetooth device reported as paired on the default adapter.

| Field | Type | Description |
|-------|------|-------------|
| `address` | `bluer::Address` | The device's Bluetooth MAC address. |
| `name` | `Option<String>` | The remote-advertised friendly name, if BlueZ has one cached. |

The name is optional because BlueZ may not have a cached friendly name for a paired device, particularly if the device has not been recently connected or if it does not advertise a name in its service records.

### `list_paired_devices`

```rust
pub async fn list_paired_devices() -> Result<Vec<PairedDevice>, DiscoverError>
```

Lists all devices paired on the default Bluetooth adapter.

**Behavior:**

- Queries the default BlueZ adapter for all known device addresses.
- Filters to only those devices where `is_paired()` returns `true`.
- Does **not** filter by current connection or reachability state. A paired device that is currently out of range or powered off is still returned.
- Returns devices in an unspecified order.

**Errors:**

| Error | Condition |
|-------|-----------|
| `DiscoverError::Bluez` | The BlueZ D-Bus session or default adapter cannot be reached, or a device's `is_paired` or `name` property cannot be read. |

## SDP Channel Resolution

### `Channels`

RFCOMM channels resolved over SDP for the MAP and PBAP services.

| Field | Type | Description |
|-------|------|-------------|
| `map` | `Option<u8>` | MAP RFCOMM channel. `None` means the device has no MAP service record. |
| `pbap` | `Option<u8>` | PBAP RFCOMM channel. `None` means the device has no PBAP service record. |

A `None` field indicates the absence of a service record, not a failure. Devices may support MAP but not PBAP, or neither.

### `resolve_channels`

```rust
pub async fn resolve_channels(address: &str) -> Result<Channels, DiscoverError>
```

Resolves the MAP and PBAP RFCOMM channels reported by a device's SDP server.

**Parameters:**

- `address` — A MAC address string in the form `XX:XX:XX:XX:XX:XX`. The function accepts a string rather than `bluer::Address` because callers (such as the GUI, which receives the address from a frontend) may only have a string and should not require a `bluer` dependency to round-trip it.

**Behavior:**

- Opens a separate SDP connection for each profile.
- Queries the device's SDP server for the MAP service UUID (`0x1133`) and retrieves the RFCOMM channel from the service record.
- Queries the device's SDP server for the PBAP service UUID (`0x1134`) and retrieves the RFCOMM channel from the service record.
- Returns a `Channels` struct with `Some(channel)` for each profile that has a service record, or `None` if the service record is absent.

**Errors:**

| Error | Condition |
|-------|-----------|
| `DiscoverError::Sdp` | The address is malformed, or an SDP exchange fails (connect failure, I/O, timeout, or malformed response). |

A missing service record is **not** an error. It is reported as `None` in the returned `Channels` struct.

## Error Types

### `DiscoverError`

```rust
pub enum DiscoverError {
    Bluez(bluer::Error),
    Sdp(bluesdp::SdpError),
}
```

| Variant | Source | Description |
|---------|--------|-------------|
| `Bluez` | D-Bus operations | BlueZ/D-Bus failure during session setup, adapter lookup, or device property read. |
| `Sdp` | SDP queries | SDP exchange failure resolving a channel for a service UUID. |

## Usage Pattern

A typical workflow combines both functions:

```rust
// List all paired devices
let devices = discover::list_paired_devices().await?;

// For each device, resolve its MAP/PBAP channels
for device in &devices {
    let channels = discover::resolve_channels(&device.address.to_string()).await?;
    // channels.map and channels.pbap are Option<u8>
}
```

The caller owns all presentation and selection logic. The discovery module provides only the data.

## Relationship to RFCOMM

The resolved channel numbers are used by the [`rfcomm`](rfcomm-transport.md) module to establish connections:

- `rfcomm::connect()` takes a `bluer::Address` and a channel number to open an RFCOMM socket.
- The channel must be obtained via SDP resolution (this module) or known a priori from manual configuration.

## Platform Requirements

- A running BlueZ stack with D-Bus access.
- A default Bluetooth adapter configured in BlueZ.
- The device must be paired with the system (pairing is performed through system Bluetooth settings, not through this module).

## Testing

The `list_paired_devices` and `resolve_channels` functions require a real BlueZ adapter and are not unit-tested. Unit tests in `discover.rs` verify error message formatting and the default value of `Channels`.

See [`rfcomm::connect`](rfcomm-transport.md#connect) for the BT_CONNECTED gate behavior, which ensures the RFCOMM link is fully established before the transport is returned to callers.