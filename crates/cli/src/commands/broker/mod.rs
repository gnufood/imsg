//! Broker access for CLI commands: auto-spawn on demand, then talk to it via
//! `imsg-broker-client`.
//!
//! Split into two sub-modules by concern:
//! - [`client`] — formats `imsg-broker-client`'s structured responses for `broker status`/`stop`
//! - [`spawn`] — process management (spawn the ephemeral one-shot broker subprocess)

mod client;
mod spawn;

use std::path::Path;

use anyhow::Result;
use broker_client::send_request;
use config::Config;
use ipc::{BrokerRequest, BrokerResponse};

pub(in crate::commands) use broker_client::{connect_retry, query_persistent, query_state};
pub(in crate::commands) use client::{run_status, run_stop};

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
    send_request(addr, req).await
}

/// Sends an SMS via the broker (auto-starting if necessary), sharing `imsg-broker-client`'s
/// request-building and response-interpretation with the GUI's equivalent command.
///
/// # Errors
///
/// Returns an error if the broker cannot be started, or [`broker_client::WriteError`] if the
/// connection or the request itself fails.
pub(crate) async fn send(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    number: String,
    message: String,
) -> Result<String> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    spawn::ensure_running(cfg, device, config_path).await?;
    Ok(broker_client::send(addr, number, message).await?)
}

/// Deletes a message via the broker (auto-starting if necessary); see [`send`] for why this
/// isn't inlined at each call site.
///
/// # Errors
///
/// Returns an error if the broker cannot be started, or [`broker_client::WriteError`] if the
/// connection or the request itself fails.
pub(crate) async fn delete(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    handle: String,
    folder: String,
) -> Result<String> {
    let addr = device.unwrap_or_else(|| cfg.device.address());
    spawn::ensure_running(cfg, device, config_path).await?;
    Ok(broker_client::delete(addr, handle, folder).await?)
}
