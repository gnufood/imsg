use std::fs;
use std::io;
use std::path::PathBuf;

use interprocess::local_socket::{GenericNamespaced, Name, ToNsName as _};
use toml_edit::{DocumentMut, Item, Table};

use super::ConfigError;

/// Opens (or creates) `~/.config/imsg/imsg.toml`, sets `[section].key = value`, writes back.
///
/// All other keys and sections are preserved. Parent directories are created if absent.
///
/// # Errors
///
/// Returns [`ConfigError::Io`] when the config directory cannot be determined or on FS failure.
/// Returns [`ConfigError::Parse`] when the existing config file contains invalid TOML.
/// Returns [`ConfigError::Invalid`] when the section already exists but is not a TOML table.
fn patch_config(section: &'static str, key: &str, value: &str) -> Result<(), ConfigError> {
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

/// `$XDG_STATE_HOME/imsg/broker-{addr}.log`, falling back to `~/.local/state` then `$TMPDIR`.
///
/// Truncated on each broker start. Parallel to [`broker_abstract_name`]; `addr` ensures
/// per-device isolation. Inspect this file when the broker fails to start.
#[must_use]
pub fn broker_log_path(addr: &str) -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| {
        dirs::home_dir().unwrap_or_else(std::env::temp_dir).join(".local/state")
    });
    base.join(format!("imsg/broker-{addr}.log"))
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
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    fn broker_abstract_name_produces_valid_name() -> Result<(), ConfigError> {
        broker_abstract_name("AA:BB:CC:DD:EE:FF")?;
        broker_abstract_name("00:00:00:00:00:00")?;
        Ok(())
    }

    #[test]
    fn set_hub_key_rejects_empty() {
        let result = set_hub_key("");
        assert!(matches!(result, Err(ConfigError::Invalid { field: "hub.node_key", .. })));
    }

    #[test]
    #[serial]
    fn set_hub_key_roundtrip() {
        figment::Jail::expect_with(|jail| {
            let tmp = jail.directory().to_path_buf();
            jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
            jail.set_env("HOME", tmp.to_str().unwrap_or_default());
            let key = "fakehubkey456";
            set_hub_key(key).map_err(|e| figment::Error::from(e.to_string()))?;
            // crate::figment(None) includes DEFAULTS + user file at HOME/.config/imsg/imsg.toml
            let cfg: crate::Config = crate::figment(None).extract()?;
            assert_eq!(cfg.hub.node_key.as_deref(), Some(key));
            Ok(())
        });
    }
}
