//! `send` subcommand: record outgoing intent first, then push to device.

use std::path::Path;

use anyhow::Result;
use config::Config;
use store::Store;
use transport::iroh::Endpoint;

use ipc::BrokerRequest;

use crate::commands::{broker, conn};

/// Delegates the full send lifecycle to the broker (RFCOMM path) or to `session::outbox::send_sms`
/// (hub path). Both paths enqueue to the store, push to the device, and record the outcome.
///
/// # Errors
///
/// Returns an error if the MAP connection, store write, or push fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
    store: &Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if endpoint.is_none() {
        return match broker::call(cfg, device, config_path, BrokerRequest::Send { number, message })
            .await?
        {
            ipc::BrokerResponse::Text(s) => Ok(s),
            ipc::BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
            ipc::BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
            other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
        };
    }

    let now = session::util::now_ms();
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::outbox::send_sms(&mut client, store, &number, &message, now).await;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    result
}

/// Pushes an outgoing SMS to the device without touching the store (non-opted-in `send`).
///
/// RFCOMM path (`endpoint` is `None`): the broker answers a [`BrokerRequest::SendLive`]. Hub path: a
/// direct MAP connection via `session::outbox::push_sms`. Fire-and-forget — no outbox row, so no
/// delivery tracking or retry; a transient failure surfaces here to re-send.
///
/// # Errors
///
/// Returns an error if the broker call, MAP connection, or push fails.
pub(crate) async fn run_live(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
    config_path: Option<&Path>,
) -> Result<String> {
    if endpoint.is_none() {
        return match broker::call(
            cfg,
            device,
            config_path,
            BrokerRequest::SendLive { number, message },
        )
        .await?
        {
            ipc::BrokerResponse::Text(s) => Ok(s),
            ipc::BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
            ipc::BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
            other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
        };
    }

    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::outbox::push_sms(&mut client, &number, &message).await;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    result
}
