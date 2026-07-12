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

#[cfg(test)]
mod tests;
