//! `folders` subcommand: list the device's MAP message folders.

use std::fmt::Write as _;

use anyhow::Result;
use config::Config;
use map_core::FolderListing;

use transport::iroh::Endpoint;

use crate::commands::conn;

/// Renders a folder listing as one folder name per line; a placeholder if empty.
pub(crate) fn render(listing: &FolderListing) -> String {
    let folders = listing.folders();
    if folders.is_empty() {
        return "(no folders)".to_owned();
    }
    let mut out = String::with_capacity(folders.len().saturating_mul(16));
    for f in folders {
        let _ = writeln!(out, "{}", f.name());
    }
    out
}

/// Connects, navigates `telecom/msg`, lists the message folders, and returns them rendered.
///
/// # Errors
///
/// Returns an error if the connection or folder-listing request fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let listing = client.list_message_folders().await?;
    Ok(render(&listing))
}
