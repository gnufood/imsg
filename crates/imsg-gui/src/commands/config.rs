//! `#[tauri::command]` shims over `crate::config` — local config read/write, no broker
//! involved.

use std::path::PathBuf;

use crate::dto::ConfigDto;

use super::CommandError;

/// Loads and validates the resolved configuration.
///
/// # Errors
///
/// Returns [`CommandError`] if no config source sets `device.address`, an existing value fails
/// validation, or the layered config sources can't be read.
// `PathBuf`, not `&Path`: `#[tauri::command]` arguments are deserialized from the frontend's
// IPC call and must be owned.
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
/// Returns [`CommandError`] if `address` is not a valid `XX:XX:XX:XX:XX:XX` MAC, or the config
/// file can't be written.
// `String`, not `&str`: `#[tauri::command]` arguments are deserialized from the frontend's IPC
// call and must be owned.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub fn config_set_device(address: String) -> Result<(), CommandError> {
    Ok(crate::config::set_device(&address)?)
}

/// Persists the MAP RFCOMM channel to the user config file.
///
/// # Errors
///
/// Returns [`CommandError`] if `channel` is not in `[1, 30]`, or the config file can't be
/// written.
#[tauri::command]
#[specta::specta]
pub fn config_set_map_channel(channel: u8) -> Result<(), CommandError> {
    Ok(crate::config::set_map_channel(channel)?)
}

/// Persists the PBAP RFCOMM channel to the user config file.
///
/// # Errors
///
/// Returns [`CommandError`] if `channel` is not in `[1, 30]`, or the config file can't be
/// written.
#[tauri::command]
#[specta::specta]
pub fn config_set_pbap_channel(channel: u8) -> Result<(), CommandError> {
    Ok(crate::config::set_pbap_channel(channel)?)
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
/// Returns [`CommandError`] if `address` is not a valid MAC, either channel is outside
/// `[1, 30]`, or the config file can't be written.
// `String`, not `&str`: `#[tauri::command]` arguments are deserialized from the frontend's IPC
// call and must be owned.
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
