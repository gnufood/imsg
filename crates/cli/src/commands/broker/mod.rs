//! Broker IPC client: auto-spawn, connect, send/receive frames, and `broker status`.
//!
//! Split into two sub-modules by concern:
//! - [`client`] — IPC transport (frame encoding, abstract-socket connect, request/response)
//! - [`spawn`] — process management (spawn subprocess, connect-retry readiness probe)

mod client;
mod spawn;

use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::{BrokerRequest, BrokerResponse};

pub(in crate::commands) use client::{query_persistent, query_state, run_status, run_stop};
pub(in crate::commands) use spawn::connect_retry;

/// Sends `req` to the broker (auto-starting if necessary) and returns one response frame.
///
/// Resolves the abstract socket name from `cfg.device.address()` unless `device` overrides it.
/// `config_path` is forwarded as `--config` to the broker subprocess when set.
///
/// # Errors
///
/// Returns an error if the broker cannot be started, the connection fails, or frame
/// encoding/decoding fails.
pub(crate) async fn call(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    req: BrokerRequest,
) -> Result<BrokerResponse> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    spawn::ensure_running(cfg, device, config_path).await?;
    client::send_request(addr, req).await
}
