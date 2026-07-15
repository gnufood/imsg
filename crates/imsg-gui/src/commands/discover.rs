//! `#[tauri::command]` shims over `crate::discover`.

use super::CommandError;
use crate::dto::{ChannelsDto, PairedDeviceDto};

/// Lists devices paired on the default adapter, for the device-config gate's picker.
///
/// # Errors
///
/// Returns [`CommandError`] if the `BlueZ` D-Bus session or default adapter cannot be reached.
#[tauri::command]
#[specta::specta]
pub async fn discover_list_paired_devices() -> Result<Vec<PairedDeviceDto>, CommandError> {
    Ok(crate::discover::list_paired_devices().await?)
}

/// Resolves the MAP and PBAP RFCOMM channels `address` reports over SDP.
///
/// # Errors
///
/// Returns [`CommandError`] if `address` is malformed or the SDP exchange fails.
#[tauri::command]
#[specta::specta]
pub async fn discover_resolve_channels(address: String) -> Result<ChannelsDto, CommandError> {
    Ok(crate::discover::resolve_channels(&address).await?)
}

#[cfg(test)]
mod tests;
