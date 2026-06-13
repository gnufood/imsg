//! `get` subcommand: fetch one message by handle and print it.

use std::fmt::Write as _;

use anyhow::Result;
use config::Config;
use map_core::{BMessage, MessageStatus};

use crate::cli::{folder_of, FolderArg};
use transport::iroh::Endpoint;

use crate::commands::conn;

/// Picks the display name for a vCard: its N field, falling back to its TEL when N is blank.
const fn sender_label<'a>(name: &'a str, tel: &'a str) -> &'a str {
    if name.is_empty() {
        tel
    } else {
        name
    }
}

/// Renders a fetched message: sender, folder, read status, then the body text.
///
/// Sender resolves to the originator's name, then its number, then `(unknown)` when the
/// message carries no originator vCard. Does not fetch or mutate anything.
pub(crate) fn render(msg: &BMessage) -> String {
    let sender = msg.originator().map_or("(unknown)", |c| sender_label(&c.name, &c.tel));
    let status = match msg.status() {
        MessageStatus::Read => "read",
        MessageStatus::Unread => "unread",
    };
    let body = &msg.envelope().body.text;
    let folder = msg.folder().rsplit('/').next().unwrap_or_else(|| msg.folder());
    let mut out = String::with_capacity(body.len().saturating_add(64));
    let _ = write!(out, "From: {sender}\nFolder: {folder}\nStatus: {status}\n\n{body}");
    out
}

/// Connects, navigates to `folder`, fetches `handle`, and returns it rendered; optionally
/// marks it read as a best-effort follow-up.
///
/// The mark-read is best-effort: a status-update failure is logged as a warning and never
/// discards the fetched message, which is always returned to the caller for display.
///
/// # Errors
///
/// Returns an error if the connection, folder navigation, or fetch fails. A failed mark-read
/// is warned, not propagated.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    handle: String,
    folder: Option<FolderArg>,
    mark_read: bool,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    client.set_folder(folder_of(folder)).await?;
    let msg = client.get_message(&handle).await?;
    let out = render(&msg);
    if mark_read {
        if let Err(e) = client.set_message_status_read(&handle, MessageStatus::Read).await {
            tracing::warn!("failed to mark {handle} read: {e}");
        }
    }
    Ok(out)
}
