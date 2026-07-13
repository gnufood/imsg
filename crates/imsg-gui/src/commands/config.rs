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

#[cfg(test)]
mod tests;
