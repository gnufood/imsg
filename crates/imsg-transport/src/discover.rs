//! Bluetooth device discovery: paired-device listing and RFCOMM channel resolution over SDP.
//!
//! Shared by the CLI's `config setup` subcommand and the GUI's device-config gate screen.
//! Contains no picker/interaction logic — callers own how a device is presented and selected.

use bluesdp::Uuid16;

/// A Bluetooth device `BlueZ` reports as paired on the default adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairedDevice {
    /// The device's Bluetooth MAC address.
    pub address: bluer::Address,
    /// The remote-advertised friendly name, if `BlueZ` has one cached.
    pub name: Option<String>,
}

/// RFCOMM channels resolved over SDP for the MAP and PBAP services.
///
/// A `None` field means the device has no service record for that profile — not a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Channels {
    /// MAP RFCOMM channel.
    pub map: Option<u8>,
    /// PBAP RFCOMM channel.
    pub pbap: Option<u8>,
}

/// Everything that can go wrong listing paired devices or resolving their SDP channels.
#[derive(Debug, thiserror::Error)]
pub enum DiscoverError {
    /// `BlueZ`/D-Bus failure — session setup, adapter lookup, or a device property read.
    #[error("bluez: {0}")]
    Bluez(#[from] bluer::Error),
    /// SDP exchange failure resolving a channel for a service UUID.
    #[error("sdp: {0}")]
    Sdp(#[from] bluesdp::SdpError),
}

/// Lists devices paired on the default adapter.
///
/// Does not filter by current connection/reachability state — a paired-but-currently-out-of-range
/// device is still returned.
///
/// # Errors
///
/// Returns [`DiscoverError::Bluez`] if the `BlueZ` D-Bus session or default adapter cannot be
/// reached, or a device's `is_paired`/`name` property cannot be read.
pub async fn list_paired_devices() -> Result<Vec<PairedDevice>, DiscoverError> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let mut devices = Vec::new();
    for address in adapter.device_addresses().await? {
        let device = adapter.device(address)?;
        if device.is_paired().await? {
            devices.push(PairedDevice { address, name: device.name().await? });
        }
    }
    tracing::debug!("discover: {} paired device(s)", devices.len());
    Ok(devices)
}

/// Resolves the MAP and PBAP RFCOMM channels `address`'s SDP server reports.
///
/// Takes a plain `XX:XX:XX:XX:XX:XX` string, not [`bluer::Address`] — `bluesdp` parses and
/// validates it itself, so callers that only ever have a string (e.g. the GUI, which receives
/// one back from the frontend) don't need a `bluer` dependency just to round-trip it.
///
/// # Errors
///
/// Returns [`DiscoverError::Sdp`] if `address` is malformed, or either SDP exchange fails
/// (connect failure, I/O, timeout, or malformed response). A missing service record is not an
/// error — see [`Channels`].
pub async fn resolve_channels(address: &str) -> Result<Channels, DiscoverError> {
    let map = query_channel(address, "MAP", Uuid16::MAP).await?;
    let pbap = query_channel(address, "PBAP", Uuid16::PBAP).await?;
    Ok(Channels { map, pbap })
}

// Each profile is resolved over its own SDP connection, and a MAP failure short-circuits the
// PBAP query — so neither which of the two failed nor how far the sequence got is recoverable
// from `DiscoverError` alone. Logged per query to keep that attribution.
async fn query_channel(
    address: &str,
    profile: &str,
    service_uuid: Uuid16,
) -> Result<Option<u8>, DiscoverError> {
    tracing::debug!("sdp: querying {address} for {profile}");
    let channel = bluesdp::query_rfcomm_channel(address, service_uuid)
        .await
        .inspect_err(|e| tracing::warn!("sdp: {profile} query to {address} failed: {e}"))?;
    tracing::debug!("sdp: {address} reports {profile} channel {channel:?}");
    Ok(channel)
}

#[cfg(test)]
mod tests {
    use bluesdp::SdpError;

    use super::*;

    // `list_paired_devices`/`resolve_channels` themselves need a real BlueZ adapter and are not
    // unit-tested here, matching this crate's existing convention for hardware-backed calls
    // (see `rfcomm.rs`'s untested `connect`/`listen_mns`).

    #[test]
    fn discover_error_sdp_message_wraps_source() {
        let err = DiscoverError::from(SdpError::Decode("truncated".to_string()));
        assert_eq!(err.to_string(), "sdp: failed to decode SDP response: truncated");
    }

    #[test]
    fn channels_default_is_no_service_records() {
        assert_eq!(Channels::default(), Channels { map: None, pbap: None });
    }
}
