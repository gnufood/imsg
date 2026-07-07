//! Persistent daemon entry point: runs the device actor with idle disabled, so the broker
//! survives indefinitely with zero subscribers or `DeviceOp`s. [`super::run`] is the ephemeral
//! (CLI one-shot) counterpart; both share [`super::bind`] for the socket election.

use anyhow::Result;
use config::Config;
use store::Store;

use super::server;

/// Same socket election and lazy MAP-session establishment as [`super::run`].
///
/// Unlike `run`, the actor's idle timeout is disabled — it only stops on a fatal MAP failure or
/// an explicit stop request via the shutdown coordinator.
///
/// # Errors
///
/// Returns an error if socket binding or the accept loop fails fatally. A failed MAP connect is
/// handled by the actor (it shuts the broker down), not returned here.
pub async fn run_daemon(cfg: Config, device_override: Option<String>, store: Store) -> Result<()> {
    let (addr_str, addr, map_channel, listener) =
        super::bind(&cfg, device_override.as_deref(), false)?;
    server::serve_daemon(cfg, addr_str, addr, map_channel, store, listener).await
}
