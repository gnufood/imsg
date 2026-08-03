//! Broker status/stop commands: format `imsg-broker-client`'s structured responses for display.

use std::time::Duration;

use anyhow::Result;
use broker_client::{probe, send_request};
use config::Config;
use ipc::{BrokerRequest, BrokerResponse};

/// Bound on how long `run_stop` waits, after a successful `Shutdown`, for the socket to become
/// unreachable — bridges the daemon's own bounded drain so callers see a clean "not running"
/// instead of racing a socket that answered `Ok` but hasn't actually torn down its listener yet.
const STOP_CONFIRM_BOUND: Duration = Duration::from_secs(5);

/// Returns a one-line health summary, or `"not running"` if the broker is unreachable.
///
/// `label` prefixes the summary (e.g. `"broker"` or `"daemon"`) to match the command the
/// caller invoked — the underlying probe is identical regardless of which spawned the process.
///
/// Does not start the broker — only probes whether it is already listening.
///
/// # Errors
///
/// Returns an error if a running broker returns a malformed response.
pub(in crate::commands) async fn run_status(
    cfg: &Config,
    device: Option<&str>,
    label: &str,
) -> Result<String> {
    let addr = super::resolve_addr(cfg, device);
    let Ok(resp) = send_request(addr, BrokerRequest::Status).await else {
        return Ok(format!("{label} for {addr}: not running"));
    };
    match resp {
        BrokerResponse::StatusInfo { state, device: dev, .. } => {
            Ok(format!("{label} for {dev}: {state}"))
        }
        other => Ok(format!("unexpected response: {other:?}")),
    }
}

/// Sends a graceful `Shutdown` request, or reports `"not running"` if unreachable.
///
/// Idempotent by design — stopping something that isn't running is a no-op, not an error, and
/// this never auto-starts the broker just to shut it back down. On a successful `Shutdown`, waits
/// (bounded by [`STOP_CONFIRM_BOUND`]) for the socket to actually go unreachable before returning,
/// so a command run immediately after `stop` doesn't race a daemon that answered but hasn't
/// finished tearing down its listener.
///
/// # Errors
///
/// Returns an error if the connection succeeds but sending or receiving the frame fails.
pub(in crate::commands) async fn run_stop(cfg: &Config, device: Option<&str>) -> Result<String> {
    let addr = super::resolve_addr(cfg, device);
    let Ok(resp) = send_request(addr, BrokerRequest::Shutdown).await else {
        return Ok(format!("daemon for {addr}: not running"));
    };
    match resp {
        BrokerResponse::Ok => {
            await_unreachable(addr).await;
            Ok(format!("daemon for {addr}: stopping"))
        }
        BrokerResponse::Error(e) => Ok(format!("daemon for {addr}: {e}")),
        other => Ok(format!("unexpected response: {other:?}")),
    }
}

/// Polls `addr` until nothing answers a raw connect, or [`STOP_CONFIRM_BOUND`] elapses.
async fn await_unreachable(addr: &str) {
    let Some(deadline) = tokio::time::Instant::now().checked_add(STOP_CONFIRM_BOUND) else {
        return;
    };
    while tokio::time::Instant::now() < deadline {
        if !probe(addr).await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}
