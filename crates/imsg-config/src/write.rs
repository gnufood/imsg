use std::fs;
use std::io;
use std::path::PathBuf;

use toml_edit::{DocumentMut, Item, Table};

use super::ConfigError;

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
    if root.get("device").is_none_or(|i| !i.is_table()) {
        root.insert("device", Item::Table(Table::new()));
    }
    let device =
        root.get_mut("device").and_then(Item::as_table_mut).ok_or(ConfigError::Invalid {
            field: "device",
            msg: "[device] section is not a TOML table".to_owned(),
        })?;
    device.insert("address", toml_edit::value(address));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, doc.to_string())?;
    Ok(())
}

/// `{data_dir}/imsg/hub.key`; `None` in minimal containers where `dirs::data_dir()` is unavailable.
/// Callers must not substitute a fallback silently.
#[must_use]
pub fn hub_key_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("imsg/hub.key"))
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
    if root.get("hub").is_none_or(|i| !i.is_table()) {
        root.insert("hub", Item::Table(Table::new()));
    }
    let hub = root.get_mut("hub").and_then(Item::as_table_mut).ok_or(ConfigError::Invalid {
        field: "hub",
        msg: "[hub] section is not a TOML table".to_owned(),
    })?;
    hub.insert("node_key", toml_edit::value(key));

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, doc.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

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
