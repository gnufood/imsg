# Configuration Options

imsg uses a layered configuration system that merges values from multiple sources. The configuration controls device identification, RFCOMM channel assignments, hub connectivity, database storage, and broker session timing.

## Configuration Layering

Configuration is loaded from sources in ascending priority order:

1. **Compiled-in defaults** — hardcoded values in the binary
2. **`/etc/imsg.toml`** — system-wide configuration
3. **`~/.config/imsg/imsg.toml`** — user-specific configuration (XDG)
4. **`./imsg.toml`** — local configuration in the working directory
5. **Explicit path** — passed programmatically to `load()`
6. **Environment variables** — prefixed with `IMSG_`, using `__` as the nesting separator

Missing files are silently skipped. Environment variables override all file-based sources.

### Environment Variable Format

Environment variables use double underscores to separate nested keys:

```sh
IMSG_DEVICE__ADDRESS=AA:BB:CC:DD:EE:FF
IMSG_DEVICE__MAP_CHANNEL=5
IMSG_BROKER__IDLE_SECS=30
```

## Device Configuration

The `[device]` section specifies the target Bluetooth device and its RFCOMM channels.

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `address` | string | Valid MAC address (`XX:XX:XX:XX:XX:XX`) | **required** |
| `map_channel` | u8 | Integer in [1, 30] | 2 |
| `pbap_channel` | u8 | Integer in [1, 30], must differ from `map_channel` | 13 |

### MAC Address Format

The device address must be a valid Bluetooth MAC address in colon-separated hexadecimal format:

```
AA:BB:CC:DD:EE:FF
```

Addresses are validated using `bluer::Address::parse()` at load time.

### RFCOMM Channel Constraints

- Both channels must be integers in the range [1, 30]
- `map_channel` and `pbap_channel` must be different values — the device cannot route inbound data to two profiles on the same RFCOMM channel

### Writing Device Configuration

The following functions write to `~/.config/imsg/imsg.toml`, creating the file and parent directories if absent:

- `set_device(address: &str)` — writes only the MAC address
- `set_channels(map_channel: u8, pbap_channel: u8)` — writes both channels atomically
- `set_device_and_channels(address: &str, map_channel: u8, pbap_channel: u8)` — writes address and both channels atomically

All write functions validate inputs before performing any I/O. A rejected value never leaves a partially-written configuration.

## Hub Configuration

The `[hub]` section contains the iroh node key for connecting to the hub.

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `node_key` | Option\<string\> | Non-empty string when present | absent |

The `node_key` is written by the `imsg spoke add` command. It is not validated at load time — validation occurs at connect time via `key.parse::<EndpointId>()`.

### Writing Hub Configuration

`set_hub_key(key: &str)` writes the node key to the user config file. The key must be non-empty.

## Store Configuration

The `[store]` section specifies the database location.

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `path` | Option\<PathBuf\> | Absolute path when present | `db_path()` fallback |

When `path` is absent, `StoreConfig::resolve()` falls back to the default database path.

### Default Database Path

`db_path()` returns `{data_dir}/imsg/messages.db`, where `data_dir` is determined by `dirs::data_dir()`. Returns `None` in minimal container environments.

## Broker Configuration

The `[broker]` section controls session broker lifecycle and startup timing.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `idle_secs` | u64 | 15 | Seconds of inactivity after which the broker disconnects MAP and exits |
| `connect_max_attempts` | u32 | 3 | Maximum MAP session connection attempts (must be >= 1) |
| `bt_connected_secs` | u64 | 5 | Per-attempt timeout for the RFCOMM `BT_CONNECTED` gate |
| `initial_backoff_ms` | u64 | 500 | Initial backoff between MAP connect attempts (milliseconds) |
| `max_backoff_secs` | u64 | 30 | Backoff ceiling between MAP connect attempts |
| `startup_budget_secs` | u64 | 30 | Total wall-clock budget for establishing the session |
| `readiness_wait_secs` | u64 | 40 | CLI deadline awaiting broker readiness |
| `readiness_poll_ms` | u64 | 50 | CLI poll interval while awaiting broker readiness |
| `security_level` | Option\<SecurityLevel\> | None | Minimum RFCOMM security level for MAP socket |

### Timing Budget

The broker timing fields form a single budget:

- A startup may make up to `connect_max_attempts` connection attempts
- Each attempt is gated by `bt_connected_secs` for RFCOMM connectivity
- Attempts are separated by exponential backoff starting at `initial_backoff_ms`, doubling up to `max_backoff_secs`
- The total budget is bounded by `startup_budget_secs`
- The CLI waits `readiness_wait_secs` (polling every `readiness_poll_ms`) for the broker to become ready

### Validation Constraint

`readiness_wait_secs` must exceed `startup_budget_secs` by at least 2 seconds (the IPC margin). This ensures the CLI never gives up while the broker is still legitimately connecting.

### Security Level

The optional `security_level` field requests a specific RFCOMM security level from the kernel for the MAP connect socket:

| Value | Kernel Constant | Description |
|-------|-----------------|-------------|
| `sdp` | `BT_SECURITY_SDP` | SDP-only traffic, no security |
| `low` | `BT_SECURITY_LOW` | No encryption or authentication required |
| `medium` | `BT_SECURITY_MEDIUM` | Encryption required; no authentication |
| `high` | `BT_SECURITY_HIGH` | Encryption and authentication required (MITM protection) |

When absent (the default), imsg never requests a security level — the kernel's already-negotiated pairing/bond security applies unmodified.

### Duration Accessors

`BrokerConfig` provides duration accessors that convert the raw integer fields to `Duration` objects:

- `idle()` → `Duration`
- `bt_connected()` → `Duration`
- `initial_backoff()` → `Duration`
- `max_backoff()` → `Duration`
- `startup_budget()` → `Duration`
- `readiness_wait()` → `Duration`
- `readiness_poll()` → `Duration`

## Path Functions

The following functions resolve paths used by imsg components:

| Function | Returns | Description |
|----------|---------|-------------|
| `hub_key_path()` | Option\<PathBuf\> | `{data_dir}/imsg/hub.key` — iroh node key storage |
| `db_path()` | Option\<PathBuf\> | `{data_dir}/imsg/messages.db` — default database location |
| `hub_lock_path()` | Option\<PathBuf\> | `{data_dir}/imsg/hub.lock` — advisory lock file |
| `broker_abstract_name(addr: &str)` | io::Result\<Name\<'static\>\> | Abstract socket name for the broker serving a device |
| `broker_log_path(addr: &str)` | PathBuf | `$XDG_STATE_HOME/imsg/broker-{addr}.log` |
| `daemon_log_path(addr: &str)` | PathBuf | `$XDG_STATE_HOME/imsg/daemon-{addr}.log` |

The broker abstract name uses Linux's abstract socket namespace, providing per-device isolation without a separate registry. The `addr` component (Bluetooth MAC) ensures each device gets its own socket.

Log paths fall back to `~/.local/state` then `$TMPDIR` when `$XDG_STATE_HOME` is unavailable.

## Errors

Configuration operations may return the following error variants:

- **`ConfigError::Load`** — figment failed to extract configuration (missing required field or type mismatch)
- **`ConfigError::Invalid`** — domain constraint violated (invalid MAC address, out-of-range channel, timing inconsistency)
- **`ConfigError::Io`** — filesystem or I/O failure
- **`ConfigError::Parse`** — existing config file contains invalid TOML

## Loading Configuration

```rust
use imsg_config::{load, Config};

let config = load(None)?;
// config.device.address(), config.device.map_channel, etc.
```

The `load()` function extracts configuration from all layers, validates domain constraints, and returns a `Config` struct.

To check whether a device is configured without triggering errors on other missing fields:

```rust
use imsg_config::is_device_configured;

if is_device_configured(None) {
    // Device address is set in some configuration layer
}
```

## See Also

- [Configuration file format specification](./config-layering.md)
- [Daemon startup flow](../broker-daemon/index.md)
- [Device pairing and SDP discovery](../transport/device-discovery.md)