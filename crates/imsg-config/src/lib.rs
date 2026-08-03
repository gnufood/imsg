//! Layered configuration for imsg: compiled-in defaults → /etc → XDG user → local → env.

mod broker;
mod write;

pub use broker::{BrokerConfig, SecurityLevel};
pub use write::{
    broker_abstract_name, broker_log_path, daemon_log_path, db_path, hub_key_path, hub_lock_path,
    set_broker_security_level, set_device, set_device_and_channels, set_hub_key, set_map_channel,
    set_pbap_channel,
};

use std::path::PathBuf;

use figment::providers::{Env, Format, Toml};
use figment::Figment;
use serde::Deserialize;

// compiled-in defaults. device.address intentionally absent — required from caller.
// no [hub] defaults — hub.node_key is absent until `imsg spoke add` writes it.
const DEFAULTS: &str = r"
[device]
map_channel = 2
pbap_channel = 13

[broker]
idle_secs = 15
connect_max_attempts = 3
bt_connected_secs = 5
initial_backoff_ms = 500
max_backoff_secs = 30
startup_budget_secs = 30
readiness_wait_secs = 40
readiness_poll_ms = 50
";

/// Layers (ascending priority): compiled-in defaults → `/etc/imsg.toml` →
/// `~/.config/imsg/imsg.toml` → `./imsg.toml` → explicit path (if any) → `IMSG_` env vars
/// using `__` as the nesting separator. File layers are silently skipped when absent.
#[must_use]
pub(crate) fn figment(explicit: Option<PathBuf>) -> Figment {
    let mut f = Figment::from(Toml::string(DEFAULTS)).merge(Toml::file("/etc/imsg.toml"));

    if let Some(xdg) = dirs::config_dir() {
        f = f.merge(Toml::file(xdg.join("imsg/imsg.toml")));
    }

    let mut f = f.merge(Toml::file("imsg.toml"));

    if let Some(path) = explicit {
        f = f.merge(Toml::file(path));
    }

    f.merge(Env::prefixed("IMSG_").split("__"))
}

/// Call at startup before any I/O.
///
/// # Errors
///
/// Returns [`ConfigError::Load`] when a required field is absent or has the wrong type.
/// Returns [`ConfigError::Invalid`] when `device.address` is not a valid MAC address or a
/// channel is outside `[1, 30]`.
pub fn load(explicit: Option<PathBuf>) -> Result<Config, ConfigError> {
    let cfg: Config = figment(explicit).extract()?;
    validate(&cfg)?;
    Ok(cfg)
}

/// `true` if `device.address` is set by any layered source (file/env), without validating its
/// value or erroring on any other missing/malformed field.
///
/// For callers (the GUI's device-config startup gate) that need "no device configured yet"
/// distinguished from every other [`load()`] failure — [`ConfigError::Load`] doesn't carry
/// structured field information, so pattern-matching it can't make that distinction.
#[must_use]
pub fn is_device_configured(explicit: Option<PathBuf>) -> bool {
    figment(explicit).contains("device.address")
}

/// Errors from [`load()`] and [`set_device()`] — figment extraction, domain validation, I/O, and TOML parse.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Figment failed to extract — missing required field or type mismatch; inner error names the key.
    #[error("failed to load config: {0}")]
    Load(Box<figment::Error>),
    /// Domain constraint violated — `field` names the dotted key, `msg` describes the violation.
    #[error("invalid config: {field}: {msg}")]
    Invalid {
        /// Dotted config key path where the constraint was violated (e.g. `"device.address"`).
        field: &'static str,
        /// Free-form description of the constraint violation.
        msg: String,
    },
    /// Reading or writing the config file failed.
    #[error("config I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The existing config file contains invalid TOML and cannot be safely edited.
    #[error("config TOML parse error: {0}")]
    Parse(#[from] toml_edit::TomlError),
}

impl From<figment::Error> for ConfigError {
    fn from(e: figment::Error) -> Self {
        Self::Load(Box::new(e))
    }
}

/// Construct via [`load()`].
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Address and RFCOMM channels; validated on [`load()`].
    pub device: DeviceConfig,
    /// Absent until `imsg spoke add` writes the key.
    #[serde(default)]
    pub hub: HubConfig,
    /// Absent from config — `store.resolve()` falls back to [`db_path`].
    #[serde(default)]
    pub store: StoreConfig,
    /// Session-broker lifecycle and startup-timing policy; see [`BrokerConfig`].
    #[serde(default)]
    pub broker: BrokerConfig,
}

/// MAC address (`XX:XX:XX:XX:XX:XX`) plus MAP/PBAP RFCOMM channels `[1, 30]`.
#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    address: String,
    /// RFCOMM channel for the MAP MAS profile. Validated to `[1, 30]`.
    pub map_channel: u8,
    /// RFCOMM channel for the PBAP PSE profile. Validated to `[1, 30]`.
    pub pbap_channel: u8,
}

impl DeviceConfig {
    /// Guaranteed valid `XX:XX:XX:XX:XX:XX` format after a successful [`load()`].
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }
}

/// Absent until `imsg spoke add <KEY>` writes it. Unvalidated — checked at connect time via `node_key.parse::<EndpointId>()`.
#[derive(Debug, Default, Deserialize)]
pub struct HubConfig {
    /// The iroh public key written by `imsg spoke add`.
    pub node_key: Option<String>,
}

/// Optional DB path override. Absent from config → [`StoreConfig::resolve`] falls back to [`db_path`].
#[derive(Debug, Default, Deserialize)]
pub struct StoreConfig {
    /// Absolute path to the `SQLCipher` database file. When absent, resolved via [`db_path`].
    pub(crate) path: Option<PathBuf>,
}

impl StoreConfig {
    /// Returns the configured path when set; otherwise delegates to [`db_path`].
    ///
    /// Returns `None` only in minimal containers where [`db_path`] itself returns `None`.
    #[must_use]
    pub fn resolve(&self) -> Option<PathBuf> {
        self.path.clone().or_else(db_path)
    }
}

/// Checks `device.address` MAC format, channel values in `[1, 30]`, and broker timing
/// consistency. Does not cross-validate bridge addresses against each other or verify device
/// reachability.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] when `device.address` is not `XX:XX:XX:XX:XX:XX`, a channel
/// value is `0` or greater than `30`, or the `[broker]` timing policy is inconsistent.
pub(crate) fn validate(cfg: &Config) -> Result<(), ConfigError> {
    cfg.device
        .address
        .parse::<bluer::Address>()
        .map_err(|e| ConfigError::Invalid { field: "device.address", msg: e.to_string() })?;
    validate_channel("device.map_channel", cfg.device.map_channel)?;
    validate_channel("device.pbap_channel", cfg.device.pbap_channel)?;
    cfg.broker.validate()
}

/// Shared by [`validate`] (load-time) and `write::{set_map_channel,set_pbap_channel}`
/// (pre-write, same bound so a saved value never fails the next load).
pub(crate) fn validate_channel(field: &'static str, channel: u8) -> Result<(), ConfigError> {
    if channel == 0 || channel > 30 {
        return Err(ConfigError::Invalid { field, msg: format!("{channel} is not in [1, 30]") });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
