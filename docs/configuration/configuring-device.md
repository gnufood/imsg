# Configuring a Device

This page traces the flow of persisting a Bluetooth device's MAC address and RFCOMM channels to the user configuration file. The flow begins after the device has been discovered through Bluetooth SDP queries and ends with the configuration being available for subsequent `load()` operations.

## Operation Overview

The operation writes three values to the user config file at `~/.config/imsg/imsg.toml`:
- `device.address`: The Bluetooth MAC address in `XX:XX:XX:XX:XX:XX` format
- `device.map_channel`: The RFCOMM channel for the MAP (Message Access Profile) MAS role
- `device.pbap_channel`: The RFCOMM channel for the PBAP (Phone Book Access Profile) PSE role

The primary entry point is `set_device_and_channels()`, which performs all three writes in a single atomic read-modify-write cycle. This atomicity is critical: the configuration schema requires all three fields, so partial writes would leave the config in an unloadable state.

## Participating Components

| Component | Role |
|-----------|------|
| `write::set_device_and_channels()` | Primary API for combined device+channel persistence |
| `write::patch_config()` | Low-level read-modify-write engine |
| `lib::validate_channel()` | Channel bounds validation (1-30) |
| `lib::validate_channel_pair()` | Ensures MAP and PBAP channels differ |
| `bluer::Address` | MAC address parsing and validation |
| `toml_edit` | TOML document manipulation |

## Flow Phases

### Phase 1: Input Validation

The caller invokes `set_device_and_channels(address, map_channel, pbap_channel)` with the discovered values. The function performs all validation before any filesystem I/O occurs—this fail-fast approach ensures no partial state is written if validation fails.

**Address validation** (lines 122-124 in `write.rs`):
```rust
address
    .parse::<bluer::Address>()
    .map_err(|e| ConfigError::Invalid { field: "device.address", msg: e.to_string() })?;
```

The MAC address must parse successfully as a `bluer::Address`. Invalid formats (wrong length, invalid hex characters, wrong separators) return `ConfigError::Invalid` with the field set to `"device.address"`.

**Channel bounds validation** (lines 125-126 in `write.rs`):
```rust
crate::validate_channel("device.map_channel", map_channel)?;
crate::validate_channel("device.pbap_channel", pbap_channel)?;
```

Each channel must be in the range `[1, 30]`. Channel 0 is rejected because RFCOMM channels are 1-indexed. Channel values above 30 are rejected because the Bluetooth specification defines only channels 1-30 as usable for RFCOMM.

**Channel pair validation** (line 127 in `write.rs`):
```rust
crate::validate_channel_pair(map_channel, pbap_channel)?;
```

The MAP and PBAP channels must differ. The Bluetooth device cannot route inbound data to two profiles on the same RFCOMM channel simultaneously. This validation uses the `pbap_channel` field name in the error, but the message explains the constraint.

If any validation fails, the function returns early with `ConfigError::Invalid` and no filesystem modification occurs.

### Phase 2: Config File Preparation

Once validation passes, `patch_config()` is called with the section name `"device"` and an array of key-value pairs (lines 129-136 in `write.rs`):
```rust
patch_config(
    "device",
    &[
        ("address", address.into()),
        ("map_channel", i64::from(map_channel).into()),
        ("pbap_channel", i64::from(pbap_channel).into()),
    ],
)
```

The `patch_config()` function (lines 23-55 in `write.rs`) determines the user config path by querying `dirs::config_dir()` and appending `"imsg/imsg.toml"`. This follows the XDG Base Directory Specification, typically resolving to `~/.config/imsg/imsg.toml`.

**File existence handling** (lines 32-36 in `write.rs`):
```rust
let mut doc: DocumentMut = match fs::read_to_string(&path) {
    Ok(content) => content.parse()?,
    Err(e) if e.kind() == io::ErrorKind::NotFound => DocumentMut::new(),
    Err(e) => return Err(ConfigError::Io(e)),
};
```

If the config file exists, it is parsed as TOML. If it does not exist, a new empty document is created. Any other I/O error (permission denied, disk full, etc.) propagates as `ConfigError::Io`.

**TOML parse errors** (line 33 in `write.rs`):
```rust
Ok(content) => content.parse()?,
```

If the existing file contains invalid TOML, parsing fails with `toml_edit::TomlError`, which is wrapped as `ConfigError::Parse`. This prevents corrupting an already-parseable config file with partial writes.

### Phase 3: Document Modification

The function inserts or updates the `[device]` section and writes the three values (lines 38-48 in `write.rs`):
```rust
let root = doc.as_table_mut();
if root.get(section).is_none_or(|i| !i.is_table()) {
    root.insert(section, Item::Table(Table::new()));
}
let table = root.get_mut(section).and_then(Item::as_table_mut).ok_or(ConfigError::Invalid {
    field: section,
    msg: format!("[{section}] section is not a TOML table"),
})?;
for (key, value) in entries {
    table.insert(key, toml_edit::value(value.clone()));
}
```

The code ensures the `[device]` section exists as a TOML table, then iterates through the provided key-value pairs, inserting or updating each. All three values are written in a single pass—this is the atomicity guarantee that `set_device_and_channels()` provides.

### Phase 4: Directory Creation and File Write

Before writing, the function ensures the parent directory exists (lines 50-53 in `write.rs`):
```rust
if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
}
fs::write(&path, doc.to_string())?;
```

`create_dir_all()` is idempotent—it succeeds if the directory already exists. The final `fs::write()` persists the modified TOML document. Both operations can fail with `ConfigError::Io` if the filesystem is not writable or the disk is full.

## Alternative Entry Points

The crate provides three public functions for device configuration, each serving different use cases:

| Function | Use Case |
|----------|----------|
| `set_device_and_channels()` | Combined write after device discovery; atomic across all three values |
| `set_device()` | Write only the MAC address (channels may come from defaults or env) |
| `set_channels()` | Update channels independently (address already configured) |

The `set_channels()` function (lines 88-100 in `write.rs`) validates and writes only the two channel values. It is useful for UI-driven settings changes where the user modifies channels without re-selecting the device.

The `set_device()` function (lines 68-73 in `write.rs`) writes only the address. This is less common but useful when the channels are already correct and only the device needs to be changed.

## Failure Paths

| Failure Condition | Error Type | Recovery |
|-------------------|------------|----------|
| Invalid MAC format | `ConfigError::Invalid` (field: `device.address`) | Caller must obtain correct address |
| Channel out of bounds (0 or >30) | `ConfigError::Invalid` (field: `device.map_channel` or `device.pbap_channel`) | Caller must obtain valid channel from SDP |
| MAP channel equals PBAP channel | `ConfigError::Invalid` (field: `device.pbap_channel`) | Caller must select different channels |
| Config file contains invalid TOML | `ConfigError::Parse` | User must fix or delete corrupted file |
| Config directory not writable | `ConfigError::Io` | User must fix permissions |
| Disk full | `ConfigError::Io` | User must free disk space |

## Resulting State

After a successful call to `set_device_and_channels()`, the user config file contains:

```toml
[device]
address = "AA:BB:CC:DD:EE:FF"
map_channel = 2
pbap_channel = 13
```

The configuration is now available to subsequent `load()` calls. The layered config system merges this user config file (at `~/.config/imsg/imsg.toml`) with compiled-in defaults, `/etc/imsg.toml`, and environment variables, with the user file taking precedence over all other sources except an explicit path argument.

The device address becomes required for all subsequent operations—the broker cannot start without knowing which Bluetooth device to connect to, and the daemon uses the address to derive its abstract socket name for IPC isolation.

## Related Documentation

- [Configuration Loading](./configuration-loading.md) — How `load()` merges and validates the layered configuration
- [Broker Configuration](./broker-configuration.md) — Session broker lifecycle and timing policy
- [Error Types](./config-error.md) — Detailed `ConfigError` variants and their causes