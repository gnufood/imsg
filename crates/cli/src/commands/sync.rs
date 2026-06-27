//! `sync` subcommand — one-shot backfill of the local store from the device.

use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::BrokerRequest;
use map_core::folders::Folder;
use store::Store;
use transport::iroh::Endpoint;

use crate::cli::FolderArg;
use crate::commands::{broker, conn};

/// Backfills MAP folders since their per-folder cursor anchors and returns a completion message.
///
/// Hub path: direct MAP connection. RFCOMM path: broker IPC.
/// Sets `sync_enabled = "true"` on success.
///
/// # Errors
///
/// Returns an error if the MAP connection, any MAP fetch, or any store write fails.
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
        let req = BrokerRequest::Sync { folder: folder_name };
        return match broker::call(cfg, device, config_path, req).await? {
            ipc::BrokerResponse::Text(s) => {
                store.set_meta("sync_enabled", "true").await?;
                Ok(s)
            }
            ipc::BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
            ipc::BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
            other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
        };
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
    store.set_meta("sync_enabled", "true").await?;
    Ok("sync complete".to_owned())
}
