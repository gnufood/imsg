//! `sync` subcommand — one-shot backfill of the local store from the device.

use std::path::Path;

use anyhow::Result;
use config::Config;
use map_core::folders::Folder;
use store::Store;
use transport::iroh::Endpoint;

use crate::cli::FolderArg;
use crate::commands::{broker, conn, contacts};

/// Backfills MAP folders since their per-folder cursor anchors, refreshes the contacts cache,
/// and returns a completion message. Sets `sync_enabled = "true"` on success.
///
/// Hub path: direct MAP connection, then a direct PBAP connection for contacts. RFCOMM path:
/// broker IPC for both. The contacts step is not best-effort — same strict `?` propagation as
/// the MAP steps, so a PBAP failure fails the whole command, same as a MAP failure would.
///
/// # Errors
///
/// Returns an error if the MAP connection, any MAP fetch, the contacts refresh, or any store
/// write fails.
pub(crate) async fn run(
    cfg: &Config,
    spoke: Option<&Endpoint>,
    device: Option<&str>,
    store: &Store,
    folder: Option<FolderArg>,
    config_path: Option<&Path>,
) -> Result<String> {
    let folder_name = folder.map(|f| {
        let folder_enum = match f {
            FolderArg::Inbox => Folder::Inbox,
            FolderArg::Sent => Folder::Sent,
            FolderArg::Outbox => Folder::Outbox,
            FolderArg::Deleted => Folder::Deleted,
        };
        folder_enum.as_str().to_owned()
    });

    if spoke.is_none() {
        let text = broker::sync(cfg, device, config_path, folder_name).await?;
        let contacts_count = contacts::run_sync(cfg, spoke, device, store, config_path).await?;
        store.set_meta("sync_enabled", "true").await?;
        return Ok(format!("{text}; {contacts_count} contacts synced"));
    }

    // Hub path: direct MAP connection.
    let folder_scope = folder.map(|f| match f {
        FolderArg::Inbox => Folder::Inbox,
        FolderArg::Sent => Folder::Sent,
        FolderArg::Outbox => Folder::Outbox,
        FolderArg::Deleted => Folder::Deleted,
    });
    let mut client = conn::connect_map(cfg, spoke, device).await?;
    let now = session::util::now_ms();
    session::outbox::drain_outbox(&mut client, store, now).await?;
    session::sync::backfill(&mut client, store, folder_scope).await?;
    let contacts_count = contacts::run_sync(cfg, spoke, device, store, config_path).await?;
    store.set_meta("sync_enabled", "true").await?;
    Ok(format!("sync complete; {contacts_count} contacts synced"))
}
