//! Ephemeral broker process management: spawn the one-shot broker subprocess on demand.

use std::path::Path;

use anyhow::Result;
use broker_client::{connect_retry, probe};
use config::Config;

/// Ensures the broker abstract socket for `device` is connectable, spawning it if needed.
///
/// If the abstract name is already connectable returns immediately. Otherwise spawns the
/// broker and enters a connect-retry loop that probes every 25 ms for up to 5 s, racing
/// against premature child exit. No startup handshake frames are exchanged — a successful
/// connect is the sole readiness signal (the kernel queues the connection in the listen
/// backlog until the broker begins accepting).
///
/// # Errors
///
/// Returns an error if spawning fails, the child exits before the socket becomes
/// connectable, or the 5 s deadline expires without a successful connect.
pub(super) async fn ensure_running(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
) -> Result<()> {
    let addr = super::resolve_addr(cfg, device);

    if probe(addr).await {
        tracing::debug!("broker: reusing already-running broker/daemon for {addr}");
        return Ok(());
    }

    tracing::debug!("broker: none reachable for {addr}, spawning ephemeral broker");
    let log_path = config::broker_log_path(addr);
    let mut child =
        imsg_proc::respawn_self(&["__broker_serve"], addr, config_path, &log_path, false, false)
            .await?;
    connect_retry(
        addr,
        &mut child,
        &log_path,
        cfg.broker.readiness_wait(),
        cfg.broker.readiness_poll(),
    )
    .await
}
