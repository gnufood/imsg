//! Local configuration read/write — pure `imsg-config` calls, no broker or device connection.

use std::path::PathBuf;

use crate::dto::ConfigDto;

/// Loads and validates the resolved configuration.
///
/// # Errors
///
/// See [`config::load`].
pub fn show(explicit: Option<PathBuf>) -> Result<ConfigDto, config::ConfigError> {
    Ok(ConfigDto::from(&config::load(explicit)?))
}

/// Persists `address` to the user config file (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// See [`config::set_device`].
pub fn set_device(address: &str) -> Result<(), config::ConfigError> {
    config::set_device(address)
}

/// Persists `map_channel` and `pbap_channel` together to the user config file
/// (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// See [`config::set_channels`].
pub fn set_channels(map_channel: u8, pbap_channel: u8) -> Result<(), config::ConfigError> {
    config::set_channels(map_channel, pbap_channel)
}

/// Persists the RFCOMM `BT_SECURITY` requirement to the user config file
/// (`~/.config/imsg/imsg.toml`).
///
/// # Errors
///
/// See [`config::set_broker_security_level`].
pub fn set_broker_security_level(level: config::SecurityLevel) -> Result<(), config::ConfigError> {
    config::set_broker_security_level(level)
}

/// `true` if the user has configured a device address yet.
///
/// [`show`] can't distinguish "no device configured yet" from any other config error.
#[must_use]
pub fn is_device_configured() -> bool {
    config::is_device_configured(None)
}

/// Persists `address`, `map_channel`, and `pbap_channel` together.
///
/// A resolved channel has no "not found" representation in config; see
/// [`config::set_device_and_channels`].
///
/// # Errors
///
/// See [`config::set_device_and_channels`].
pub fn set_device_and_channels(
    address: &str,
    map_channel: u8,
    pbap_channel: u8,
) -> Result<(), config::ConfigError> {
    config::set_device_and_channels(address, map_channel, pbap_channel)
}

#[cfg(test)]
mod tests;
