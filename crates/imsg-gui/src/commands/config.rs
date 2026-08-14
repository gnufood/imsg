//! `#[tauri::command]` shims over `crate::config` — local config read/write, no broker
//! involved.

use std::path::PathBuf;

use crate::dto::{ConfigDto, SecurityLevelDto};

use super::CommandError;

/// Loads and validates the resolved configuration.
///
/// # Errors
///
/// Returns [`CommandError`] if no config source sets `device.address`, or validation fails.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub fn config_show(config_path: Option<PathBuf>) -> Result<ConfigDto, CommandError> {
    Ok(crate::config::show(config_path)?)
}

/// Persists `address` to the user config file.
///
/// # Errors
///
/// Returns [`CommandError`] if `address` is not a valid MAC, or the write fails.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub fn config_set_device(address: String) -> Result<(), CommandError> {
    Ok(crate::config::set_device(&address)?)
}

/// Persists the MAP and PBAP RFCOMM channels together to the user config file.
///
/// # Errors
///
/// Returns [`CommandError`] if a channel is outside `[1, 30]`, the two are equal, or the write
/// fails.
#[tauri::command]
#[specta::specta]
pub fn config_set_channels(map_channel: u8, pbap_channel: u8) -> Result<(), CommandError> {
    Ok(crate::config::set_channels(map_channel, pbap_channel)?)
}

/// Persists the RFCOMM `BT_SECURITY` requirement to the user config file.
///
/// # Errors
///
/// Returns [`CommandError`] on filesystem failure, or if the existing config can't be parsed.
#[tauri::command]
#[specta::specta]
pub fn config_set_broker_security_level(level: SecurityLevelDto) -> Result<(), CommandError> {
    Ok(crate::config::set_broker_security_level(level.into())?)
}

/// `true` if the user has configured a device address yet — the device-config startup gate's
/// pre-check.
#[tauri::command]
#[specta::specta]
#[must_use]
pub fn config_is_device_configured() -> bool {
    crate::config::is_device_configured()
}

/// Persists `address`, `map_channel`, and `pbap_channel` together, e.g. after the device-config
/// gate's discovery flow resolves them.
///
/// # Errors
///
/// Returns [`CommandError`] if `address` is not a valid MAC, a channel is outside `[1, 30]`,
/// the two are equal, or the write fails.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub fn config_set_device_and_channels(
    address: String,
    map_channel: u8,
    pbap_channel: u8,
) -> Result<(), CommandError> {
    Ok(crate::config::set_device_and_channels(&address, map_channel, pbap_channel)?)
}

#[cfg(test)]
mod tests;
