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
    // Logged at the boundary as well as per-query in `transport::discover` — this is the only
    // point that knows a resolution attempt was user-initiated and that its failure reached the
    // frontend, so a log records one line per click. The gate picker and Settings' "Detect from
    // device" share this command, so it cannot say which of the two asked.
    tracing::debug!("sdp: resolution requested for {address}");
    Ok(crate::discover::resolve_channels(&address)
        .await
        .inspect_err(|e| tracing::warn!("sdp: resolution for {address} failed: {e}"))?)
}

#[cfg(test)]
mod tests;
