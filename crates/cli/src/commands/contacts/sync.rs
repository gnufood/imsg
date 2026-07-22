//! Refreshes the contacts cache.
//!
//! Local/non-hub via the broker, hub via a direct PBAP connection using the shared
//! `session::contacts::sync_contacts`. Always the main phonebook, regardless of `--path`
//! (matches the broker's `do_sync_contacts`). Shared by the `--sync` flag and `imsg sync`'s
//! contacts step.

use anyhow::Result;
use config::Config;
use pbap_core::phonebook::PhonebookPath;
use store::Store;
use transport::iroh::Endpoint;

use crate::commands::{broker, conn};

/// Refreshes the local contacts cache from the device, returning the number of address rows
/// upserted.
///
/// # Errors
///
/// Returns an error if the broker call or the PBAP connection/sync fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    store: &Store,
    config_path: Option<&std::path::Path>,
) -> Result<usize> {
    if endpoint.is_none() {
        return broker::sync_contacts(cfg, device, config_path).await;
    }
    let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
    let count = session::contacts::sync_contacts(&mut client, store, PhonebookPath::Pb).await?;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("PBAP disconnect failed: {e}");
    }
    Ok(count)
}
