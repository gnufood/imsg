//! `config` subcommand handlers.

use std::path::PathBuf;

use anyhow::{Context, Result};

/// Returns it rendered for display.
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
