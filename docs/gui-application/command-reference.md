# Command Reference

The Tauri frontend communicates with the Rust backend through IPC commands registered via `tauri_specta`. This reference documents all commands exposed to the frontend, organized by domain.

## Command Registration

All commands are registered in `commands.rs` using `tauri_specta::collect_commands!`:

```rust
pub fn builder<R: tauri::Runtime>() -> tauri_specta::Builder<R> {
    tauri_specta::Builder::<R>::new()
        .commands(tauri_specta::collect_commands![
            // reads
            reads::get_by_handle,
            reads::list_messages,
            reads::mark_read,
            reads::threads,
            // contacts
            contacts::list_contacts,
            contacts::get_contact,
            contacts::lookup_contact,
            contacts::sync_contacts_now,
            // config
            config::config_show,
            config::config_set_device,
            config::config_set_channels,
            config::config_is_device_configured,
            config::config_set_device_and_channels,
            config::config_set_broker_security_level,
            // discover
            discover::discover_list_paired_devices,
            discover::discover_resolve_channels,
            // gate
            gate::gate_status,
            gate::gate_proceed,
            // daemon
            daemon::daemon_install,
            daemon::daemon_uninstall,
            daemon::daemon_status,
            daemon::daemon_service_status,
            daemon::daemon_stop,
            daemon::daemon_restart,
            daemon::broker_status,
            // send
            send::send,
            // delete
            delete::delete,
        ])
        // ...
}
```

## Error Handling

All commands return `Result<T, CommandError>` on the frontend. `CommandError` wraps underlying errors from various subsystems:

- `store::Error` — local SQLite store failures
- `config::ConfigError` — configuration validation or file I/O failures
- `service::Error` — OS service manager failures
- `broker_client::WriteError` / `broker_client::ContactsError` — broker IPC failures
- `transport::discover::DiscoverError` — Bluetooth discovery failures
- `crate::daemon::StopError` — daemon shutdown failures
- `crate::daemon::provision::ProvisionError` — daemon self-provisioning failures

The `CommandError` message contains a sanitized `Display` string that never leaks secrets (key material, SQLite internals, or service-manager specifics beyond driver-level descriptions).

---

## Reads Domain

Commands for querying the local message store. These commands access the GUI's own `Store` connection, independent of the daemon's broker connection.

### `get_by_handle`

Returns a single message by its MAP handle, or `None` if not found.

**Signature:**
```rust
pub async fn get_by_handle(
    store: State<'_, store::Store>,
    handle: String,
) -> Result<Option<MessageDto>, CommandError>
```

**Parameters:**
- `store`: Managed `Store` state
- `handle`: Opaque MAP message handle

**Returns:** `Option<MessageDto>` — the message if present

---

### `list_messages`

Returns messages matching all supplied criteria, newest-first.

**Signature:**
```rust
pub async fn list_messages(
    store: State<'_, store::Store>,
    folder: Option<String>,
    unread_only: bool,
    from: Option<String>,
    since_ms: Option<i64>,
    limit: u16,
    offset: u16,
) -> Result<Vec<MessageDto>, CommandError>
```

**Parameters:**
- `store`: Managed `Store` state
- `folder`: Filter by MAP folder (e.g., `telecom/msg/inbox`)
- `unread_only`: If `true`, return only unread messages
- `from`: Filter by sender address
- `since_ms`: Return messages newer than this Unix timestamp in milliseconds
- `limit`: Maximum messages to return
- `offset`: Number of messages to skip

**Returns:** `Vec<MessageDto>` — messages in reverse chronological order

---

### `mark_read`

Marks a message as read in the local store. Device-side mark-read is deferred to the next sync.

**Signature:**
```rust
pub async fn mark_read(
    store: State<'_, store::Store>,
    handle: String,
) -> Result<(), CommandError>
```

**Parameters:**
- `store`: Managed `Store` state
- `handle`: MAP message handle

**Errors:** Returns `CommandError` if the underlying store write fails

---

### `threads`

Returns a per-address conversation thread summary, most-recent-first.

**Signature:**
```rust
pub async fn threads(
    store: State<'_, store::Store>,
) -> Result<Vec<ThreadDto>, CommandError>
```

**Parameters:**
- `store`: Managed `Store` state

**Returns:** `Vec<ThreadDto>` — thread summaries ordered by most recent message

---

## Contacts Domain

Commands for querying cached contacts and triggering contact synchronization.

### `list_contacts`

Returns lightweight cached contact identities (UID and display name only), ordered by display name, page-limited.

**Signature:**
```rust
pub async fn list_contacts(
    store: State<'_, store::Store>,
    limit: u16,
    offset: u16,
) -> Result<Vec<ContactEntryDto>, CommandError>
```

**Parameters:**
- `store`: Managed `Store` state
- `limit`: Maximum contacts to return
- `offset`: Number of contacts to skip

**Returns:** `Vec<ContactEntryDto>` — lightweight contact entries

---

### `get_contact`

Returns the full cached contact for a given UID, including phone numbers.

**Signature:**
```rust
pub async fn get_contact(
    store: State<'_, store::Store>,
    uid: String,
) -> Result<Option<ContactDto>, CommandError>
```

**Parameters:**
- `store`: Managed `Store` state
- `uid`: Contact's vCard UID

**Returns:** `Option<ContactDto>` — full contact with phone numbers, or `None` if uncached

---

### `lookup_contact`

Returns the full cached contact that owns a given phone number.

**Signature:**
```rust
pub async fn lookup_contact(
    store: State<'_, store::Store>,
    address: String,
) -> Result<Option<ContactDto>, CommandError>
```

**Parameters:**
- `store`: Managed `Store` state
- `address`: Phone number to look up

**Returns:** `Option<ContactDto>` — contact owning the number, or `None` if unknown

---

### `sync_contacts_now`

Triggers an immediate contacts refresh via the broker. Unlike the startup gate's best-effort sync, failure is surfaced to the caller.

**Signature:**
```rust
pub async fn sync_contacts_now(
    addr: String,
) -> Result<ipc::SyncReportDto, CommandError>
```

**Parameters:**
- `addr`: Broker IPC socket address

**Returns:** `ipc::SyncReportDto` — report of what the sync accomplished

**Errors:** Returns `CommandError` if the broker can't be reached or rejects the request

---

## Config Domain

Commands for reading and writing local configuration. These operate on the user config file (`~/.config/imsg/imsg.toml`) without involving the broker.

### `config_show`

Loads and validates the resolved configuration from all config sources.

**Signature:**
```rust
pub fn config_show(
    config_path: Option<PathBuf>,
) -> Result<ConfigDto, CommandError>
```

**Parameters:**
- `config_path`: Optional explicit config file path

**Returns:** `ConfigDto` — resolved configuration

**Errors:** Returns `CommandError` if no config source sets `device.address`, validation fails, or config files can't be read

---

### `config_set_device`

Persists a Bluetooth device address to the user config file.

**Signature:**
```rust
pub fn config_set_device(
    address: String,
) -> Result<(), CommandError>
```

**Parameters:**
- `address`: MAC address in `XX:XX:XX:XX:XX:XX` format

**Errors:** Returns `CommandError` if `address` is not a valid MAC or the config file can't be written

---

### `config_set_channels`

Persists the MAP and PBAP RFCOMM channels together to the user config file in a single operation.

**Signature:**
```rust
pub fn config_set_channels(
    map_channel: u8,
    pbap_channel: u8,
) -> Result<(), CommandError>
```

**Parameters:**
- `map_channel`: MAP RFCOMM channel number in `[1, 30]`
- `pbap_channel`: PBAP RFCOMM channel number in `[1, 30]`

**Errors:** Returns `CommandError` if either channel is outside `[1, 30]` or the config file can't be written

---

### `config_set_broker_security_level`

Persists the RFCOMM `BT_SECURITY` requirement to the user config file.

**Signature:**
```rust
pub fn config_set_broker_security_level(
    level: SecurityLevelDto,
) -> Result<(), CommandError>
```

**Parameters:**
- `level`: Security level (`Sdp`, `Low`, `Medium`, `High`)

**Errors:** Returns `CommandError` on filesystem failure or if the config file can't be parsed

---

### `config_is_device_configured`

Returns `true` if the user has configured a device address. This is a cheap pre-check for the device-config startup gate.

**Signature:**
```rust
pub fn config_is_device_configured() -> bool
```

**Returns:** `bool` — `true` if a device address is configured

---

### `config_set_device_and_channels`

Persists device address, MAP channel, and PBAP channel together in a single operation.

**Signature:**
```rust
pub fn config_set_device_and_channels(
    address: String,
    map_channel: u8,
    pbap_channel: u8,
) -> Result<(), CommandError>
```

**Parameters:**
- `address`: MAC address in `XX:XX:XX:XX:XX:XX` format
- `map_channel`: MAP RFCOMM channel in `[1, 30]`
- `pbap_channel`: PBAP RFCOMM channel in `[1, 30]`

**Errors:** Returns `CommandError` if the address is invalid, any channel is outside `[1, 30]`, or the config file can't be written

---

## Daemon Domain

Commands for daemon lifecycle management: installation, status queries, stop, and restart.

### `daemon_install`

Registers the daemon with the native OS service manager.

**Signature:**
```rust
pub fn daemon_install(
    addr: String,
    config_path: Option<PathBuf>,
    system: bool,
) -> Result<(), CommandError>
```

**Parameters:**
- `addr`: Broker IPC socket address
- `config_path`: Optional explicit config file path
- `system`: If `true`, install as a system service; otherwise as a user service

**Errors:** Returns `CommandError` if no native service manager is available or it rejects the install

---

### `daemon_uninstall`

Unregisters the daemon service, reporting whether anything was there to remove.

**Signature:**
```rust
pub fn daemon_uninstall(
    system: bool,
) -> Result<UninstallResult, CommandError>
```

**Parameters:**
- `system`: If `true`, uninstall the system service; otherwise the user service

**Returns:** `UninstallResult` — either `NotInstalled` or `Uninstalled`

**Errors:** Returns `CommandError` if no native service manager is available or it rejects the uninstall

---

### `daemon_status`

Returns the daemon's current session state, or `None` if nothing answers at the address.

**Signature:**
```rust
pub async fn daemon_status(
    addr: String,
) -> Option<SessionState>
```

**Parameters:**
- `addr`: Broker IPC socket address

**Returns:** `Option<SessionState>` — session state (`Initializing`, `Active`, `Failed`, `ShuttingDown`) or `None` if unreachable

---

### `daemon_service_status`

Returns the daemon service's registration and run state at the given level.

**Signature:**
```rust
pub fn daemon_service_status(
    system: bool,
) -> Result<ServiceInstallState, CommandError>
```

**Parameters:**
- `system`: If `true`, check the system service; otherwise the user service

**Returns:** `ServiceInstallState` — one of:
- `NotInstalled` — no service registered
- `Running` — registered and running
- `Stopped(Option<String>)` — registered but not running

**Errors:** Returns `CommandError` if no native service manager is available or it fails to report status

---

### `broker_status`

Returns the broker's current session state. This is the same query as `daemon_status` — the CLI distinguishes ephemeral vs. persistent, but the query itself has no domain-specific shape.

**Signature:**
```rust
pub async fn broker_status(
    addr: String,
) -> Option<SessionState>
```

**Parameters:**
- `addr`: Broker IPC socket address

**Returns:** `Option<SessionState>` — session state or `None` if unreachable

---

### `daemon_stop`

Sends a graceful shutdown request to the daemon.

**Signature:**
```rust
pub async fn daemon_stop(
    addr: String,
) -> Result<StopOutcome, CommandError>
```

**Parameters:**
- `addr`: Broker IPC socket address

**Returns:** `StopOutcome` — either `NotRunning` (nothing was reachable) or `Stopping` (shutdown accepted)

**Errors:** Returns `CommandError` if the broker answers but rejects the request or returns an unexpected response

---

### `daemon_restart`

Ensures a daemon is reachable at the given address, self-provisioning one if none answers. This is the GUI's only way to bring the daemon back after `daemon_stop`, since `main.rs`'s self-provisioning only runs once at startup.

**Signature:**
```rust
pub async fn daemon_restart(
    addr: String,
    config_path: Option<PathBuf>,
) -> Result<(), CommandError>
```

**Parameters:**
- `addr`: Broker IPC socket address
- `config_path`: Optional explicit config file path

**Errors:** Returns `CommandError` if the address is held by an ephemeral broker, the config can't be loaded, or spawning fails

---

## Discover Domain

Commands for Bluetooth device discovery and SDP channel resolution, used by the device-config startup gate.

### `discover_list_paired_devices`

Lists devices paired on the default Bluetooth adapter.

**Signature:**
```rust
pub async fn discover_list_paired_devices() -> Result<Vec<PairedDeviceDto>, CommandError>
```

**Returns:** `Vec<PairedDeviceDto>` — paired devices with address and optional name

**Errors:** Returns `CommandError` if the BlueZ D-Bus session or default adapter cannot be reached

---

### `discover_resolve_channels`

Resolves the MAP and PBAP RFCOMM channels for a given device address via SDP.

**Signature:**
```rust
pub async fn discover_resolve_channels(
    address: String,
) -> Result<ChannelsDto, CommandError>
```

**Parameters:**
- `address`: Bluetooth MAC address

**Returns:** `ChannelsDto` — with `map: Option<u8>` and `pbap: Option<u8>`. A `None` field means the device has no service record for that profile.

**Errors:** Returns `CommandError` if the address is malformed or the SDP exchange fails

---

## Gate Domain

Commands for interacting with the startup gate — the Rust-owned loop that brings the app from cold start to a ready state.

### `gate_status`

Returns the startup gate's current status. The frontend polls this until `Ready`.

**Signature:**
```rust
pub fn gate_status(
    state: State<'_, GateState>,
) -> GateStatus
```

**Parameters:**
- `state`: Managed `GateState`

**Returns:** `GateStatus` — one of:
- `Initializing` — gate hasn't reached first config check
- `AwaitingDeviceConfig` — no valid device config; parked until `gate_proceed`
- `StartingDaemon` — daemon self-provisioning or store open in flight
- `Syncing` — MAP sync in flight
- `Failed { stage, message }` — a step failed; parked until `gate_proceed`
- `Ready` — gate completed; store handed to frontend

---

### `gate_proceed`

Wakes a parked startup gate. Call after persisting device config, or as the retry action on a `Failed` status.

**Signature:**
```rust
pub fn gate_proceed(
    state: State<'_, GateState>,
)
```

**Parameters:**
- `state`: Managed `GateState`

**Behavior:** The gate re-derives everything from ground truth on each pass. Stray or repeated calls are harmless.

---

## Send Domain

### `send`

Sends an SMS via the broker's tracked-outbox path. The daemon holds the sole live MAP session, so this forwards to the broker rather than touching the GUI's own store.

**Signature:**
```rust
pub async fn send(
    addr: String,
    number: String,
    message: String,
) -> Result<String, CommandError>
```

**Parameters:**
- `addr`: Broker IPC socket address
- `number`: Recipient phone number
- `message`: SMS body text

**Returns:** `String` — the message handle on success

**Errors:** Returns `CommandError` if the broker can't be reached or rejects the request

---

## Delete Domain

### `delete`

Sets the deleted flag on the device and removes the message from the broker's store.

**Signature:**
```rust
pub async fn delete(
    addr: String,
    handle: String,
    folder: String,
) -> Result<String, CommandError>
```

**Parameters:**
- `addr`: Broker IPC socket address
- `handle`: MAP message handle
- `folder`: MAP folder containing the message

**Returns:** `String` — the message handle on success

**Errors:** Returns `CommandError` if the broker can't be reached or rejects the request

---

## Data Transfer Objects

### MessageDto

```rust
pub struct MessageDto {
    pub handle: String,           // Opaque MAP message handle
    pub timestamp_ms: i64,        // Milliseconds since Unix epoch
    pub folder: String,           // MAP folder (e.g., telecom/msg/inbox)
    pub direction: Direction,     // Received or Sent
    pub address: String,          // Remote phone number
    pub read: bool,               // true unless message is unread
    pub text: String,             // Decoded message body
    pub outgoing_status: Option<OutgoingStatus>, // For sent messages
}
```

### ThreadDto

```rust
pub struct ThreadDto {
    pub address: String,                      // Contact address
    pub latest_ms: i64,                       // Most recent message timestamp
    pub total: i64,                           // Total message count
    pub unread: i64,                          // Unread received message count
    pub latest_outgoing_status: Option<OutgoingStatus>,
    pub contact_name: Option<String>,         // Cached PBAP display name
}
```

### Direction

```rust
pub enum Direction {
    Received,  // From remote
    Sent,      // By this device
}
```

### OutgoingStatus

```rust
pub enum OutgoingStatus {
    Queued,          // Outbox entry created; push not yet attempted
    Sending,         // Push in progress
    SentUnconfirmed, // Device accepted; not yet confirmed
    SentConfirmed,   // Confirmed in Sent folder via reconciliation
    FailedRetryable, // Transient error; retry warranted
    FailedPermanent, // Permanent error; no retry
    Unknown,         // Connection dropped; outcome requires reconciliation
}
```

### ConfigDto

```rust
pub struct ConfigDto {
    pub device_address: String,           // Bluetooth MAC
    pub map_channel: u8,                  // MAP RFCOMM channel
    pub pbap_channel: u8,                 // PBAP RFCOMM channel
    pub hub_node_key: Option<String>,     // Iroh hub node key
    pub security_level: Option<SecurityLevelDto>,
}
```

### SecurityLevelDto

```rust
pub enum SecurityLevelDto {
    Sdp,    // BT_SECURITY_SDP — no security
    Low,    // BT_SECURITY_LOW — no encryption/auth
    Medium, // BT_SECURITY_MEDIUM — encryption only
    High,   // BT_SECURITY_HIGH — encryption and authentication
}
```

### PairedDeviceDto

```rust
pub struct PairedDeviceDto {
    pub address: String,      // Bluetooth MAC
    pub name: Option<String>, // Remote-advertised friendly name
}
```

### ChannelsDto

```rust
pub struct ChannelsDto {
    pub map: Option<u8>,   // MAP RFCOMM channel
    pub pbap: Option<u8>,  // PBAP RFCOMM channel
}
```

### ContactEntryDto

```rust
pub struct ContactEntryDto {
    pub uid: String,              // vCard UID
    pub display_name: Option<String>, // vCard FN property
}
```

### ContactDto

```rust
pub struct ContactDto {
    pub uid: String,
    pub display_name: Option<String>,
    pub phones: Vec<PhoneDto>,
}
```

### PhoneDto

```rust
pub struct PhoneDto {
    pub raw: String,           // Number as device reported
    pub e164: Option<String>,  // Canonical E.164 form, if resolvable
}
```