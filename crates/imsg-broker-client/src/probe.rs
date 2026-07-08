//! Reachability checks: is the broker/daemon socket connectable, and retry-until-connectable.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use interprocess::local_socket::ConnectOptions;
use tokio::process::Child;

/// Returns `true` if the abstract broker socket for `addr` is currently connectable.
pub async fn probe(addr: &str) -> bool {
    match config::broker_abstract_name(addr) {
        Ok(name) => ConnectOptions::new().name(name).connect_tokio().await.is_ok(),
        Err(_) => false,
    }
}

/// Retries connecting to the abstract broker socket for `addr` until success or failure.
///
/// Probes every `poll` interval. Succeeds when connect returns `Ok`; fails immediately when
/// `child` exits before that, or after `deadline_in` elapses.
///
/// # Errors
///
/// Returns an error if `child` exits before the socket is connectable or the deadline fires.
pub async fn connect_retry(
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

#[cfg(test)]
mod tests;
