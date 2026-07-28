//! `folders` subcommand: list the device's MAP message folders.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::FolderDto;
use transport::iroh::Endpoint;

use crate::commands::{broker, conn, live_footer};

/// Renders folder names one per line, or `(no folders)` when the device reports an empty
/// listing. Device order is preserved — MAP folder listings are ordered by the device.
pub(crate) fn render(folders: &[FolderDto]) -> String {
    if folders.is_empty() {
        return "(no folders)".to_owned();
    }
    let mut out = String::with_capacity(folders.len().saturating_mul(16));
    for f in folders {
        let _ = writeln!(out, "{}", f.name);
    }
    out
}

/// Lists the device's `telecom/msg` folders and returns them rendered.
///
/// RFCOMM path (`endpoint` is `None`): the broker answers a [`ipc::BrokerRequest::Folders`], so
/// this shares the broker's MAP session instead of opening a second RFCOMM connection to a
/// channel it already holds. Hub path: a direct MAP connection. Neither path touches the store.
///
/// # Errors
///
/// Returns an error if the broker call, MAP connection, or folder listing fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    config_path: Option<&Path>,
) -> Result<String> {
    if endpoint.is_none() {
        let rows = broker::folders(cfg, device, config_path).await?;
        return Ok(live_footer(render(&rows)));
    }
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::live::folders(&mut client).await;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    let rows: Vec<FolderDto> = result?.into_iter().map(|f| FolderDto { name: f.name }).collect();
    Ok(live_footer(render(&rows)))
}

#[cfg(test)]
mod tests;
