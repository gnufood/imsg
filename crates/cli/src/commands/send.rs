//! `send` subcommand: push an outbound SMS and report the assigned handle.

use anyhow::Result;
use config::Config;
use map_core::folders::Folder;

use transport::iroh::Endpoint;

use crate::commands::conn;

/// Success text naming the recipient and the handle the remote assigned to the sent message.
pub(crate) fn confirmation(number: &str, handle: &str) -> String {
    format!("sent to {number} (handle {handle})")
}

/// Connects, navigates to the outbox, pushes `message` to `number`, and returns the handle line.
///
/// # Errors
///
/// Returns an error if the connection, outbox navigation, or push fails, or if the remote
/// response omits the assigned handle.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    client.set_folder(Folder::Outbox).await?;
    let handle = client.push_message(&number, &message).await?;
    Ok(confirmation(&number, &handle))
}
