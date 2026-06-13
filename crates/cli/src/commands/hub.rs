//! `hub` subcommand: start the iroh hub and print the spoke connection key.

use std::fs;

use anyhow::{Context, Result};
use bytes::Bytes;
use config::Config;
use fs2::FileExt as _;
use tokio::sync::{broadcast, watch};

use crate::output;

/// Loads or creates the hub secret key, prints the node's [`transport::iroh::EndpointId`] to
/// stdout, then runs the iroh hub until Ctrl+C.
///
/// The hub proxies MAP and PBAP RFCOMM streams from connected spokes to the paired Bluetooth
/// device, and relays MNS notification events from the device out to subscribed spokes (so
/// `imsg watch --hub` receives live events). MNS relay failures degrade `watch` only — they do
/// not stop MAP/PBAP proxying or the hub.
///
/// # Errors
///
/// Returns an error if the hub lock cannot be acquired (another hub instance is running), the
/// hub key path cannot be determined, key I/O fails, the device address is invalid, the MNS
/// RFCOMM profile cannot be registered, or the iroh endpoint cannot bind.
pub(crate) async fn run(cfg: &Config) -> Result<()> {
    let lock_path =
        config::hub_lock_path().ok_or_else(|| anyhow::anyhow!("cannot determine hub lock path"))?;
    if let Some(parent) = lock_path.parent() {
        tokio::fs::create_dir_all(parent).await.context("creating hub data dir")?;
    }
    let _lock_file = tokio::task::spawn_blocking(move || -> anyhow::Result<fs::File> {
        let f = fs::File::create(&lock_path).context("opening hub lock file")?;
        f.try_lock_exclusive().map_err(|_| {
            anyhow::anyhow!(
                "another hub instance is already running on this machine — \
                 stop it with: pkill -f 'imsg hub'"
            )
        })?;
        Ok(f)
    })
    .await
    .context("hub lock task panicked")??;

    let path =
        config::hub_key_path().ok_or_else(|| anyhow::anyhow!("cannot determine hub key path"))?;
    let key = transport::iroh::load_or_create_key(&path).await.context("loading hub key")?;
    let id = key.public();
    output::line(&id.to_string())?;

    let addr = cfg.device.address().parse::<bluer::Address>().context("invalid device.address")?;

    let listener = transport::rfcomm::listen_mns().await.context("registering MNS profile")?;
    let (mns_tx, _) = broadcast::channel::<Bytes>(256);
    let (cancel_tx, cancel_rx) = watch::channel(false);

    let relay = tokio::spawn(session::run_mns_relay(listener, mns_tx.clone(), cancel_rx.clone()));
    let mut hub = tokio::spawn(transport::iroh::run_hub(
        key,
        addr,
        cfg.device.map_channel,
        cfg.device.pbap_channel,
        mns_tx,
        cancel_rx,
    ));

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            let _ = cancel_tx.send(true);
        }
        result = &mut hub => {
            let _ = cancel_tx.send(true);
            warn_on_relay_panic(relay.await);
            return result.context("hub task panicked")?.context("iroh hub error");
        }
    }

    hub.await.context("hub task panicked during shutdown")??;
    warn_on_relay_panic(relay.await);
    Ok(())
}

/// Logs a warning if the MNS relay task panicked. Relay failure is non-fatal — MAP/PBAP keep
/// working — so a panic is surfaced as a warning rather than propagated.
fn warn_on_relay_panic(joined: Result<(), tokio::task::JoinError>) {
    if let Err(e) = joined {
        tracing::warn!("MNS relay task panicked: {e}");
    }
}
