//! `config` subcommand handlers.

use std::path::PathBuf;

use anyhow::{Context, Result};

/// Loads the config and renders it for display.
///
/// # Errors
///
/// Returns an error if config loading fails — most commonly because `device.address`
/// is unset; the message names the fix (`config set-device`).
pub(crate) fn run_show(explicit: Option<PathBuf>) -> Result<String> {
    let cfg = super::load(explicit)?;
    Ok(render(&cfg))
}

/// Persists `address` to the user config.
///
/// # Errors
///
/// Returns an error if `address` is not a valid MAC or the config file cannot be written.
pub(crate) fn run_set_device(address: &str) -> Result<String> {
    ::config::set_device(address).context("writing device address to user config")?;
    Ok(format!("device.address = {address}"))
}

/// Lists paired devices, prompts for one, resolves its MAP/PBAP RFCOMM channels over SDP, and
/// persists address + both channels together.
///
/// # Errors
///
/// Returns an error if no devices are paired, the prompt can't read a selection, the SDP
/// exchange fails, or the device has no service record for MAP or PBAP — channels have no
/// "unset" representation in config, so a missing record must be resolved manually (`config
/// set-device` plus a hand-edited channel) rather than partially persisted here.
pub(crate) async fn run_setup() -> Result<String> {
    let devices =
        transport::discover::list_paired_devices().await.context("listing paired devices")?;
    if devices.is_empty() {
        anyhow::bail!("no paired devices found — pair one first, e.g. `bluetoothctl pair <MAC>`");
    }

    let labels: Vec<String> = devices
        .iter()
        .map(|d| {
            d.name
                .as_ref()
                .map_or_else(|| d.address.to_string(), |name| format!("{name} ({})", d.address))
        })
        .collect();
    let choice = dialoguer::Select::new()
        .with_prompt("Select a device")
        .items(&labels)
        .interact()
        .context("reading device selection")?;
    let address = devices.get(choice).context("selection index out of range")?.address;

    let channels = transport::discover::resolve_channels(&address.to_string())
        .await
        .context("resolving MAP/PBAP channels over SDP")?;
    let (Some(map), Some(pbap)) = (channels.map, channels.pbap) else {
        anyhow::bail!(
            "device {address} is missing a service record (map={:?}, pbap={:?}) — set the \
             address manually with `config set-device` and edit the missing channel by hand",
            channels.map,
            channels.pbap,
        );
    };

    ::config::set_device_and_channels(&address.to_string(), map, pbap)
        .context("writing device address and channels to user config")?;
    Ok(format!(
        "device.address      = {address}\n\
         device.map_channel  = {map}\n\
         device.pbap_channel = {pbap}"
    ))
}

/// Renders the resolved config as aligned `key = value` lines.
fn render(cfg: &::config::Config) -> String {
    format!(
        "device.address      = {}\n\
         device.map_channel  = {}\n\
         device.pbap_channel = {}\n\
         hub.node_key        = {}",
        cfg.device.address(),
        cfg.device.map_channel,
        cfg.device.pbap_channel,
        cfg.hub.node_key.as_deref().unwrap_or("<not set>"),
    )
}
