//! `delete` subcommand: mark a message deleted, or restore it with `--undelete`.

use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::BrokerRequest;
use store::Store;
use transport::iroh::Endpoint;

use crate::cli::{folder_of, FolderArg};
use crate::commands::{broker, conn};

/// Target message and mode for a `delete` invocation.
pub(crate) struct DeleteOpts {
    /// MAP message handle to act on.
    pub handle: String,
    /// Folder to scope the lookup to; defaults applied by [`folder_of`] when `None`.
    pub folder: Option<FolderArg>,
    /// Restore instead of delete; treated as a no-op (see [`run`]).
    pub undelete: bool,
}

/// One-line outcome text; `undelete` selects restore wording over delete wording.
pub(crate) fn confirmation(handle: &str, undelete: bool) -> String {
    let verb = if undelete { "restored" } else { "deleted" };
    format!("{verb} {handle}")
}

/// Marks a message deleted on the device and removes it from the local store.
///
/// Hub path: direct MAP connection. RFCOMM path: broker IPC.
///
/// Undelete is a no-op — iOS does not respect MAP `SetMessageStatus` with `StatusIndicator=DELETED`
/// and `StatusValue=0`, so we skip the request rather than hitting `RSP_NOT_IMPLEMENTED`.
/// The store entry is not touched on undelete for the same reason.
///
/// # Errors
///
/// Returns an error if the connection, folder navigation, status update, or store write fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: DeleteOpts,
    store: &Store,
    config_path: Option<&Path>,
) -> Result<String> {
    let DeleteOpts { handle, folder, undelete } = opts;
    if undelete {
        return Ok(confirmation(&handle, true));
    }
    if endpoint.is_some() {
        return run_direct(cfg, endpoint, device, &handle, folder, store).await;
    }
    let folder_name = folder_of(folder).as_str().to_ascii_lowercase();
    let req = BrokerRequest::Delete { handle: handle.clone(), folder: folder_name };
    match broker::call(cfg, device, config_path, req).await? {
        ipc::BrokerResponse::Text(_) => Ok(confirmation(&handle, false)),
        ipc::BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        ipc::BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// Direct MAP delete used on the hub path.
async fn run_direct(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    handle: &str,
    folder: Option<FolderArg>,
    store: &Store,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    client.set_folder(folder_of(folder)).await?;
    client.set_message_status_deleted(handle, true).await?;
    store.delete_by_handle(handle).await?;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    Ok(confirmation(handle, false))
}
