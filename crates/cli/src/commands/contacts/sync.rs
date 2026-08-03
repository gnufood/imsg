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

use super::render::{render_sync_dto, render_sync_report};
use crate::commands::{broker, conn};

/// Refreshes the local contacts cache from the device, returning the rendered outcome line.
///
/// Renders here rather than returning the report itself: the two transports answer with
/// different types (broker DTO vs. domain value) and both call sites — `contacts --sync` and
/// `imsg sync`'s contacts step — want the same sentence.
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
) -> Result<String> {
    if endpoint.is_none() {
        let report = broker::sync_contacts(cfg, device, config_path).await?;
        return Ok(render_sync_dto(&report));
    }
    let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
    let report = session::contacts::sync_contacts(&mut client, store, PhonebookPath::Pb).await?;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("PBAP disconnect failed: {e}");
    }
    Ok(render_sync_report(&report))
}
