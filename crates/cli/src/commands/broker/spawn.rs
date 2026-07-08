//! Broker process management: spawn the broker subprocess and probe for readiness via connect.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use config::Config;
use interprocess::local_socket::ConnectOptions;
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

/// Retries connecting to the abstract broker socket for `addr` until success or failure.
///
/// Probes every `poll` interval. Succeeds when connect returns `Ok`; fails immediately when the
/// child exits before that, or after `deadline_in` elapses. `deadline_in` is validated at config
/// load to exceed the broker's own startup budget, so this never gives up mid-connect.
///
/// Shared with `commands::daemon`'s detached-start readiness wait — the abstract-socket
/// election works identically regardless of which mode spawned the process.
///
/// # Errors
///
/// Returns an error if the child exits before the socket is connectable or the deadline fires.
pub(in crate::commands) async fn connect_retry(
    addr: &str,
    child: &mut Child,
    log_path: &Path,
    deadline_in: Duration,
    poll: Duration,
) -> Result<()> {
    let deadline = tokio::time::Instant::now()
        .checked_add(deadline_in)
        .context("startup readiness deadline overflowed")?;
    loop {
        tokio::select! {
            biased;
            () = tokio::time::sleep_until(deadline) => {
                return Err(anyhow::anyhow!(
                    "broker did not become reachable within {}s — see log: {}",
                    deadline_in.as_secs(),
                    log_path.display()
                ));
            }
            _ = child.wait() => {
                return Err(anyhow::anyhow!(
                    "broker exited during startup — see log: {}",
                    log_path.display()
                ));
            }
            () = tokio::time::sleep(poll) => {}
        }
        if probe(addr).await {
            return Ok(());
        }
    }
}

/// Returns `true` if the abstract broker socket for `addr` is currently connectable.
///
/// Shared with `commands::daemon` — "is it running" is the same check regardless of mode.
pub(in crate::commands) async fn probe(addr: &str) -> bool {
    match config::broker_abstract_name(addr) {
        Ok(name) => ConnectOptions::new().name(name).connect_tokio().await.is_ok(),
        Err(_) => false,
    }
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName as _};

    use super::connect_retry;

    /// `connect_retry` succeeds once the abstract socket becomes connectable.
    ///
    /// A background task binds the socket after 100 ms — the retry loop must discover
    /// it within the 5 s deadline without a mock or a startup handshake.
    #[tokio::test]
    async fn connect_retry_reaches_deferred_listener() -> anyhow::Result<()> {
        tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let name = "imsg/broker/FE:ED:DE:AD:00:03".to_ns_name::<GenericNamespaced>()?;
            let _l = ListenerOptions::new().name(name).create_tokio()?;
            // Keep listener alive long enough for the retry loop to connect.
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok::<(), anyhow::Error>(())
        });

        let log = PathBuf::from("/dev/null");
        let mut child = tokio::process::Command::new("sleep").arg("10").spawn()?;

        connect_retry(
            "FE:ED:DE:AD:00:03",
            &mut child,
            &log,
            Duration::from_secs(5),
            Duration::from_millis(25),
        )
        .await?;
        let _ = child.kill().await;
        Ok(())
    }

    /// `connect_retry` returns `Err` immediately when the child exits before binding.
    ///
    /// `true` exits with code 0 instantly; the socket `FE:ED:DE:AD:00:04` is never bound,
    /// so the only outcome is the child-exit arm of the `select!`.
    #[tokio::test]
    async fn connect_retry_fails_on_broker_exit() -> anyhow::Result<()> {
        let log = PathBuf::from("/dev/null");
        let mut child = tokio::process::Command::new("true").spawn()?;

        let result = connect_retry(
            "FE:ED:DE:AD:00:04",
            &mut child,
            &log,
            Duration::from_secs(5),
            Duration::from_millis(25),
        )
        .await;
        let Err(err) = result else {
            return Err(anyhow::anyhow!(
                "connect_retry should fail when the broker exits before binding"
            ));
        };
        let msg = err.to_string();
        assert!(
            msg.contains("exited during startup"),
            "expected 'exited during startup' in error, got: {msg}"
        );
        Ok(())
    }
}
