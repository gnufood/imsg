//! Layered configuration for imsg: compiled-in defaults → /etc → XDG user → local → env.

mod write;

pub use write::{hub_key_path, hub_lock_path, set_device, set_hub_key};

use std::path::PathBuf;

use figment::providers::{Env, Format, Toml};
use figment::Figment;
use serde::Deserialize;

/// Compiled-in defaults. `device.address` is intentionally absent — required from caller.
/// No `[hub]` defaults — `hub.node_key` is absent until `imsg spoke add` writes it.
const DEFAULTS: &str = r"
[device]
map_channel = 2
pbap_channel = 13
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
    /// Validated at connect time, not at write time.
    pub node_key: Option<String>,
}

/// Checks `device.address` MAC format and that channel values are in `[1, 30]`.
/// Does not cross-validate bridge addresses against each other or verify device reachability.
///
/// # Errors
///
/// Returns [`ConfigError::Invalid`] when `device.address` is not `XX:XX:XX:XX:XX:XX`
/// or a channel value is `0` or greater than `30`.
pub(crate) fn validate(cfg: &Config) -> Result<(), ConfigError> {
    cfg.device
        .address
        .parse::<bluer::Address>()
        .map_err(|e| ConfigError::Invalid { field: "device.address", msg: e.to_string() })?;
    for (field, channel) in [
        ("device.map_channel", cfg.device.map_channel),
        ("device.pbap_channel", cfg.device.pbap_channel),
    ] {
        if channel == 0 || channel > 30 {
            return Err(ConfigError::Invalid {
                field,
                msg: format!("{channel} is not in [1, 30]"),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    fn make_device(address: &str) -> DeviceConfig {
        DeviceConfig { address: address.to_owned(), map_channel: 2, pbap_channel: 13 }
    }

    #[test]
    fn device_address_required() {
        // Only compiled-in defaults — no device.address present.
        let result = Figment::from(Toml::string(DEFAULTS)).extract::<Config>();
        assert!(result.is_err());
    }

    #[test]
    fn validate_rejects_bad_address() {
        for bad in ["NOTAMAC", "GG:GG:GG:GG:GG:GG", "00:11:22:33:44", "00:11:22:33:44:55:66"] {
            let cfg = Config { device: make_device(bad), hub: HubConfig::default() };
            assert!(
                matches!(validate(&cfg), Err(ConfigError::Invalid { field: "device.address", .. })),
                "expected Invalid for {bad:?}"
            );
        }
    }

    #[test]
    fn validate_rejects_channel_bounds() {
        for bad_map in [0_u8, 31_u8] {
            let cfg = Config {
                device: DeviceConfig {
                    address: "AA:BB:CC:DD:EE:FF".to_owned(),
                    map_channel: bad_map,
                    pbap_channel: 13,
                },
                hub: HubConfig::default(),
            };
            assert!(
                matches!(
                    validate(&cfg),
                    Err(ConfigError::Invalid { field: "device.map_channel", .. })
                ),
                "expected Invalid for map_channel={bad_map}"
            );
        }
        let cfg = Config {
            device: DeviceConfig {
                address: "AA:BB:CC:DD:EE:FF".to_owned(),
                map_channel: 2,
                pbap_channel: 0,
            },
            hub: HubConfig::default(),
        };
        assert!(matches!(
            validate(&cfg),
            Err(ConfigError::Invalid { field: "device.pbap_channel", .. })
        ));
    }

    #[test]
    #[serial]
    fn env_override_address() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
            jail.set_env("IMSG_DEVICE__MAP_CHANNEL", "5");
            let cfg: Config = figment(None).extract()?;
            assert_eq!(cfg.device.address, "AA:BB:CC:DD:EE:FF");
            assert_eq!(cfg.device.map_channel, 5_u8);
            Ok(())
        });
    }

    #[test]
    #[serial]
    fn hub_defaults_to_none() {
        figment::Jail::expect_with(|jail| {
            let dir = jail.directory().to_string_lossy().into_owned();
            jail.set_env("HOME", &dir);
            jail.set_env("XDG_CONFIG_HOME", &dir);
            jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
            let cfg: Config = figment(None).extract()?;
            assert!(cfg.hub.node_key.is_none());
            Ok(())
        });
    }

    #[test]
    #[serial]
    fn env_override_hub_node_key() {
        figment::Jail::expect_with(|jail| {
            jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
            jail.set_env("IMSG_HUB__NODE_KEY", "testkey123");
            let cfg: Config = figment(None).extract()?;
            assert_eq!(cfg.hub.node_key.as_deref(), Some("testkey123"));
            Ok(())
        });
    }

    #[test]
    fn hub_key_path_format() {
        if let Some(p) = hub_key_path() {
            assert!(p.ends_with("imsg/hub.key"), "unexpected path: {p:?}");
        }
        // None is valid in container environments — silently skip.
    }

    #[test]
    fn hub_lock_path_format() {
        if let Some(p) = hub_lock_path() {
            assert!(p.ends_with("imsg/hub.lock"), "unexpected path: {p:?}");
        }
        // None is valid in container environments — silently skip.
    }
}
