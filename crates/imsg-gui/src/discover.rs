//! `discover` — paired-device listing and SDP channel resolution for the device-config gate.
//!
//! Wraps `imsg-transport::discover`. Returns GUI-local DTOs, never that crate's own types (see
//! [`crate::dto::PairedDeviceDto`]/[`crate::dto::ChannelsDto`]).

use crate::dto::{ChannelsDto, PairedDeviceDto};

/// Lists devices paired on the default adapter.
///
/// # Errors
///
/// Returns [`transport::discover::DiscoverError::Bluez`] if the `BlueZ` D-Bus session or
/// default adapter cannot be reached, or a device's properties cannot be read.
pub async fn list_paired_devices(
) -> Result<Vec<PairedDeviceDto>, transport::discover::DiscoverError> {
    let devices = transport::discover::list_paired_devices().await?;
    Ok(devices.iter().map(PairedDeviceDto::from).collect())
}

/// Resolves the MAP and PBAP RFCOMM channels `address` reports over SDP.
///
/// `address` is a plain `XX:XX:XX:XX:XX:XX` string — round-tripped through JSON from the
/// frontend's picked [`crate::dto::PairedDeviceDto`] — forwarded as-is; `bluesdp` (via
/// `transport::discover::resolve_channels`) owns parsing and validation.
///
/// # Errors
///
/// Returns [`transport::discover::DiscoverError::Sdp`] if `address` is malformed or the SDP
/// exchange fails.
pub async fn resolve_channels(
    address: &str,
) -> Result<ChannelsDto, transport::discover::DiscoverError> {
    let channels = transport::discover::resolve_channels(address).await?;
    Ok(ChannelsDto::from(channels))
}
