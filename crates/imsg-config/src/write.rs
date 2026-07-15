use std::fs;
use std::io;
use std::path::PathBuf;

use interprocess::local_socket::{GenericNamespaced, Name, ToNsName as _};
use toml_edit::{DocumentMut, Item, Table};

use super::ConfigError;

/// Opens (or creates) `~/.config/imsg/imsg.toml`, sets `[section].key = value`, writes back.
///
/// All other keys and sections are preserved. Parent directories are created if absent.
/// `value` is written with its real TOML type (e.g. a bare integer for `impl Into<Value>`'s
/// `i64` impl) — callers must pass the type the config struct actually deserializes into, or
/// the next load will fail (a quoted string never coerces into a typed numeric field).
///
/// # Errors
///
/// Returns [`ConfigError::Io`] when the config directory cannot be determined or on FS failure.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
/// Returns [`ConfigError::Invalid`] when the section already exists but is not a TOML table.
fn patch_config(
    section: &'static str,
    key: &str,
    value: impl Into<toml_edit::Value>,
) -> Result<(), ConfigError> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "cannot determine user config directory")
    })?;
    let path = config_dir.join("imsg/imsg.toml");

    let mut doc: DocumentMut = match fs::read_to_string(&path) {
        Ok(content) => content.parse()?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => DocumentMut::new(),
        Err(e) => return Err(ConfigError::Io(e)),
    };

    let root = doc.as_table_mut();
    if root.get(section).is_none_or(|i| !i.is_table()) {
        root.insert(section, Item::Table(Table::new()));
    }
    root.get_mut(section)
        .and_then(Item::as_table_mut)
        .ok_or(ConfigError::Invalid {
            field: section,
            msg: format!("[{section}] section is not a TOML table"),
        })?
        .insert(key, toml_edit::value(value));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, doc.to_string())?;
    Ok(())
}

/// Target: `~/.config/imsg/imsg.toml` (XDG). Creates the file and parent directories if absent.
///
/// All other keys preserved. Does not touch `/etc/imsg.toml`, `./imsg.toml`, or env vars.
/// Validates `address` against `XX:XX:XX:XX:XX:XX` before any I/O.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] when `address` is not a valid Bluetooth MAC.
/// Returns [`ConfigError::Io`] on filesystem failure or when the user config directory
/// cannot be determined.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
pub fn set_device(address: &str) -> Result<(), ConfigError> {
    address
        .parse::<bluer::Address>()
        .map_err(|e| ConfigError::Invalid { field: "device.address", msg: e.to_string() })?;
    patch_config("device", "address", address)
}

/// Target: `~/.config/imsg/imsg.toml` (XDG). Creates the file and parent directories if absent.
///
/// All other keys preserved. Validates `channel` is in `[1, 30]` before any I/O.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] when `channel` is `0` or greater than `30`.
/// Returns [`ConfigError::Io`] on filesystem failure or when the user config directory
/// cannot be determined.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
pub fn set_map_channel(channel: u8) -> Result<(), ConfigError> {
    crate::validate_channel("device.map_channel", channel)?;
    patch_config("device", "map_channel", i64::from(channel))
}

/// Target: `~/.config/imsg/imsg.toml` (XDG). Creates the file and parent directories if absent.
///
/// All other keys preserved. Validates `channel` is in `[1, 30]` before any I/O.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] when `channel` is `0` or greater than `30`.
/// Returns [`ConfigError::Io`] on filesystem failure or when the user config directory
/// cannot be determined.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
pub fn set_pbap_channel(channel: u8) -> Result<(), ConfigError> {
    crate::validate_channel("device.pbap_channel", channel)?;
    patch_config("device", "pbap_channel", i64::from(channel))
}

/// Target: `~/.config/imsg/imsg.toml` (XDG). Creates the file and parent directories if absent.
///
/// Writes `address`, `map_channel`, and `pbap_channel` together — the combined write automatic
/// device discovery needs, since a resolved channel has no "not found" representation in config
/// (the schema requires `u8`, not `Option<u8>`); callers must resolve or reject a missing SDP
/// record themselves before calling this. All three inputs are validated before any I/O, so a
/// rejected channel never leaves a partially-written `address`.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] if `address` is not a valid Bluetooth MAC, or if either
/// channel is `0` or greater than `30`.
/// Returns [`ConfigError::Io`] on filesystem failure or when the user config directory cannot
/// be determined.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
pub fn set_device_and_channels(
    address: &str,
    map_channel: u8,
    pbap_channel: u8,
) -> Result<(), ConfigError> {
    address
        .parse::<bluer::Address>()
        .map_err(|e| ConfigError::Invalid { field: "device.address", msg: e.to_string() })?;
    crate::validate_channel("device.map_channel", map_channel)?;
    crate::validate_channel("device.pbap_channel", pbap_channel)?;

    set_device(address)?;
    set_map_channel(map_channel)?;
    set_pbap_channel(pbap_channel)
}

/// `{data_dir}/imsg/hub.key`; `None` in minimal containers where `dirs::data_dir()` is unavailable.
/// Callers must not substitute a fallback silently.
#[must_use]
pub fn hub_key_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("imsg/hub.key"))
}

/// `{data_dir}/imsg/messages.db`; `None` in minimal containers where `dirs::data_dir()` is unavailable.
/// Callers must not substitute a fallback silently.
#[must_use]
pub fn db_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("imsg/messages.db"))
}

/// Abstract-namespace local socket name for the broker serving `addr`.
///
/// On Linux this maps to the kernel abstract socket namespace — no filesystem inode,
/// automatic kernel cleanup on process death, and `EADDRINUSE` on a second bind (atomic
/// single-instance election with no TOCTOU window).  The `addr` component (Bluetooth MAC)
/// gives per-device isolation without a separate registry.
///
/// # Errors
///
/// Returns an error if the OS rejects the derived name, which should not occur for valid
/// Bluetooth MAC strings.
pub fn broker_abstract_name(addr: &str) -> io::Result<Name<'static>> {
    format!("imsg/broker/{addr}").to_ns_name::<GenericNamespaced>()
}

/// `$XDG_STATE_HOME/imsg/{kind}-{addr}.log`, falling back to `~/.local/state` then `$TMPDIR`.
fn state_log_path(kind: &str, addr: &str) -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| {
        dirs::home_dir().unwrap_or_else(std::env::temp_dir).join(".local/state")
    });
    base.join(format!("imsg/{kind}-{addr}.log"))
}

/// `$XDG_STATE_HOME/imsg/broker-{addr}.log`, falling back to `~/.local/state` then `$TMPDIR`.
///
/// Truncated on each broker start. Parallel to [`broker_abstract_name`]; `addr` ensures
/// per-device isolation. Inspect this file when the broker fails to start.
#[must_use]
pub fn broker_log_path(addr: &str) -> PathBuf {
    state_log_path("broker", addr)
}

/// `$XDG_STATE_HOME/imsg/daemon-{addr}.log`, falling back to `~/.local/state` then `$TMPDIR`.
///
/// Truncated on each `imsg daemon start`. Distinct from [`broker_log_path`] so a persistent
/// daemon's detached stdout/stderr never mixes with an ephemeral broker session's log.
#[must_use]
pub fn daemon_log_path(addr: &str) -> PathBuf {
    state_log_path("daemon", addr)
}

/// `{data_dir}/imsg/hub.lock`. Zero-byte file used as an advisory `flock` lock.
///
/// Released on exit or crash (kernel closes all fds). Callers create and lock it —
/// this fn only resolves the path. `None` in minimal containers.
#[must_use]
pub fn hub_lock_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("imsg/hub.lock"))
}

/// Target: `~/.config/imsg/imsg.toml` (XDG). Creates the file if absent; preserves all other keys.
///
/// Does not validate `key` as an iroh `PublicKey` — deferred to connect time via
/// `key.parse::<transport::iroh::EndpointId>()`.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] if `key` is empty.
/// Returns [`ConfigError::Io`] if the user config directory cannot be determined or on
/// filesystem failure.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
pub fn set_hub_key(key: &str) -> Result<(), ConfigError> {
    if key.is_empty() {
        return Err(ConfigError::Invalid {
            field: "hub.node_key",
            msg: "must not be empty".to_owned(),
        });
    }
    patch_config("hub", "node_key", key)
}

#[cfg(test)]
mod tests;
