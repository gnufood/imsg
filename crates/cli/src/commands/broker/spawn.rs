//! Ephemeral broker process management: spawn the one-shot broker subprocess on demand.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};
use broker_client::{connect_retry, probe};
use config::Config;
use tokio::process::{Child, Command};

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
    let addr = device.unwrap_or_else(|| cfg.device.address());

    if probe(addr).await {
        tracing::debug!("broker: reusing already-running broker/daemon for {addr}");
        return Ok(());
    }

    tracing::debug!("broker: none reachable for {addr}, spawning ephemeral broker");
    let log_path = config::broker_log_path(addr);
    let mut child = spawn(cfg, device, config_path, &log_path).await?;
    connect_retry(
        addr,
        &mut child,
        &log_path,
        cfg.broker.readiness_wait(),
        cfg.broker.readiness_poll(),
    )
    .await
}

/// Spawns the broker as a detached subprocess via `current_exe()` with the hidden
/// `__broker_serve` subcommand. Stderr is redirected to `log_path` (truncated on each
/// start). Returns the child handle so the caller can race startup against premature exit.
///
/// # Errors
///
/// Returns an error if the executable path cannot be resolved, the log file cannot be
/// created, or the subprocess fails to spawn.
async fn spawn(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    log_path: &Path,
) -> Result<Child> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    let exe = std::env::current_exe().context("resolving current executable path")?;

    if let Some(parent) = log_path.parent() {
        tokio::fs::create_dir_all(parent).await.context("creating broker log directory")?;
    }
    let mut open_opts = tokio::fs::OpenOptions::new();
    open_opts.create(true).write(true).truncate(true);
    #[cfg(unix)]
    open_opts.mode(0o600); // broker log may carry message content — keep it off-limits to other users
    let log_file = open_opts
        .open(log_path)
        .await
        .with_context(|| format!("opening broker log: {}", log_path.display()))?
        .into_std()
        .await;

    let mut cmd = Command::new(exe);
    cmd.arg("__broker_serve");
    cmd.args(["--device", addr]);
    if let Some(p) = config_path {
        cmd.args(["--config", p.to_str().context("config path is not valid UTF-8")?]);
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::from(log_file));
    cmd.spawn().context("spawning broker subprocess")
}
