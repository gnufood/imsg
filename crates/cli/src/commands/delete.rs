//! `delete` subcommand: mark a message deleted, or restore it with `--undelete`.

use anyhow::Result;
use config::Config;

use crate::cli::{folder_of, FolderArg};
use transport::iroh::Endpoint;

use crate::commands::conn;

/// One-line outcome text; `undelete` selects restore wording over delete wording.
pub(crate) fn confirmation(handle: &str, undelete: bool) -> String {
    let verb = if undelete { "restored" } else { "deleted" };
    format!("{verb} {handle}")
}

/// Connects, navigates to `folder`, sets the deleted flag (`!undelete`), and returns the outcome.
///
/// # Errors
///
/// Returns an error if the connection, folder navigation, or status update fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    handle: String,
    folder: Option<FolderArg>,
    undelete: bool,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    client.set_folder(folder_of(folder)).await?;
    client.set_message_status_deleted(&handle, !undelete).await?;
    Ok(confirmation(&handle, undelete))
}
