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
use interprocess::local_socket::tokio::Stream as LocalStream;
use ipc::{BrokerRequest, BrokerResponse};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub(in crate::commands) use client::{run_status, send_frame};

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

/// Returns a live `Framed` connection to the broker for streaming requests (watch mode).
///
/// Auto-starts the broker if absent. `config_path` is forwarded as `--config` when set.
///
/// # Errors
///
/// Returns an error if the broker cannot be started or the connection fails.
pub(crate) async fn connect(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
) -> Result<Framed<LocalStream, LengthDelimitedCodec>> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    spawn::ensure_running(cfg, device, config_path).await?;
    client::connect_raw(addr).await
}
