//! Local configuration read/write — pure `imsg-config` calls, no broker or MAP/PBAP connection
//! involved (mirrors the CLI's `config show`/`set-device`, minus its anyhow-context hint text).

use std::path::PathBuf;

use crate::dto::ConfigDto;

/// Loads and validates the resolved configuration.
///
/// # Errors
///
/// Returns [`config::ConfigError`] if no config source sets `device.address`, an existing
/// value fails validation, or the layered config sources can't be read.
pub fn show(explicit: Option<PathBuf>) -> Result<ConfigDto, config::ConfigError> {
    Ok(ConfigDto::from(&config::load(explicit)?))
}

/// Persists `address` to the user config file (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// Returns [`config::ConfigError`] if `address` is not a valid `XX:XX:XX:XX:XX:XX` MAC, or the
/// config file can't be written.
pub fn set_device(address: &str) -> Result<(), config::ConfigError> {
    config::set_device(address)
}

/// Persists the MAP RFCOMM channel to the user config file (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// Returns [`config::ConfigError`] if `channel` is not in `[1, 30]`, or the config file can't
/// be written.
pub fn set_map_channel(channel: u8) -> Result<(), config::ConfigError> {
    config::set_map_channel(channel)
}

/// Persists the PBAP RFCOMM channel to the user config file (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// Returns [`config::ConfigError`] if `channel` is not in `[1, 30]`, or the config file can't
/// be written.
pub fn set_pbap_channel(channel: u8) -> Result<(), config::ConfigError> {
    config::set_pbap_channel(channel)
}

/// Persists the RFCOMM `BT_SECURITY` requirement to the user config file
/// (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// Returns [`config::ConfigError`] on filesystem failure or if the config file can't be
/// parsed.
pub fn set_broker_security_level(level: config::SecurityLevel) -> Result<(), config::ConfigError> {
    config::set_broker_security_level(level)
}

/// `true` if the user has configured a device address yet.
///
/// Cheap pre-check for the device-config startup gate — distinguishes "no device configured
/// yet" from every other config error, which [`show`] can't (see
/// [`config::is_device_configured`]).
#[must_use]
pub fn is_device_configured() -> bool {
    config::is_device_configured(None)
}

/// Persists `address`, `map_channel`, and `pbap_channel` together.
///
/// The write the device-config gate's automatic discovery flow needs, since a resolved channel
/// has no "not found" representation in config (see [`config::set_device_and_channels`]).
///
/// # Errors
///
/// Returns [`config::ConfigError`] if `address` is not a valid MAC, either channel is outside
/// `[1, 30]`, or the config file can't be written.
pub fn set_device_and_channels(
    address: &str,
    map_channel: u8,
    pbap_channel: u8,
) -> Result<(), config::ConfigError> {
    config::set_device_and_channels(address, map_channel, pbap_channel)
}

#[cfg(test)]
mod tests;
