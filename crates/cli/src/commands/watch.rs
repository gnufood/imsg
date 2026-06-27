//! `watch` subcommand — MAP notification events via stdout or ratatui panel.

use std::path::Path;

use anyhow::Result;
use config::Config;
use store::Store;
use transport::iroh::Endpoint;

use crate::commands::broker;

/// Formats one broker [`WatchEvent`] for stdout: `<EventType>  handle=<h>  folder=<f>`.
pub(crate) fn format_watch_event(ev: &ipc::WatchEvent) -> String {
    format!(
        "{}  handle={}  folder={}",
        ev.event_type,
        ev.handle.as_deref().unwrap_or("-"),
        ev.folder.as_deref().unwrap_or("-"),
    )
}

/// With `hub = true`, subscribes to the hub's MNS stream over iroh (stdout; TUI is RFCOMM-only).
/// Otherwise connects via RFCOMM and registers MNS with `BlueZ`; with `--features tui` renders
/// in ratatui. Does NOT reconnect on session drop.
///
/// # Errors
///
/// Returns error if `hub.node_key` is unset/invalid, MAP or hub connection fails, `BlueZ` MNS
/// registration fails, or the MNS task panics.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    config_path: Option<&Path>,
    store: &Store,
) -> Result<()> {
    if let Some(ep) = endpoint {
        return super::watch_hub::run(cfg, ep).await;
    }
    // RFCOMM: broker owns the MNS server and streams events over the Unix socket.
    #[cfg(feature = "tui")]
    {
        return run_tui_brokered(cfg, device, config_path, store).await;
    }
    #[cfg(not(feature = "tui"))]
    run_plain_brokered(cfg, device, config_path, store).await
}

/// Broker-backed plain watch: sends a `Watch` request, streams [`WatchEvent`] frames, prints each.
///
/// The broker owns the MNS server; this function is a pure consumer of the event stream.
///
/// # Errors
///
/// Returns an error if the broker connection or any stream frame fails.
#[cfg(not(feature = "tui"))]
async fn run_plain_brokered(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    _store: &Store,
) -> Result<()> {
    use futures::StreamExt as _;
    let mut framed = broker::connect(cfg, device, config_path).await?;
    broker::send_frame(&mut framed, &ipc::BrokerRequest::Watch).await?;

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            frame = framed.next() => match frame {
                None => break,
                Some(Err(e)) => { tracing::warn!("watch stream error: {e}"); break; }
                Some(Ok(bytes)) => {
                    let resp: ipc::BrokerResponse = serde_json::from_slice(&bytes)?;
                    match resp {
                        ipc::BrokerResponse::WatchEvent(ev) => {
                            crate::output::line(&format_watch_event(&ev))?;
                        }
                        ipc::BrokerResponse::Error(msg) => {
                            return Err(anyhow::anyhow!("broker watch error: {msg}"));
                        }
                        _ => {}
                    }
                }
            },
            _ = &mut ctrl_c => break,
        }
    }

    Ok(())
}

/// Broker-backed TUI watch. Streams [`WatchEvent`] frames from the broker and prints each.
///
/// Full ratatui TUI integration is deferred — the TUI render loop requires a live
/// `MapClient`, which is now owned by the broker process. Falls back to plain line output.
///
/// # Errors
///
/// Returns an error if the broker connection or stream fails.
#[cfg(feature = "tui")]
async fn run_tui_brokered(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    store: &Store,
) -> Result<()> {
    use futures::StreamExt as _;
    let mut framed = broker::connect(cfg, device, config_path).await?;
    broker::send_frame(&mut framed, &ipc::BrokerRequest::Watch).await?;

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            frame = framed.next() => match frame {
                None => break,
                Some(Err(e)) => { tracing::warn!("watch stream error: {e}"); break; }
                Some(Ok(bytes)) => {
                    let resp: ipc::BrokerResponse = serde_json::from_slice(&bytes)?;
                    match resp {
                        ipc::BrokerResponse::WatchEvent(ev) => {
                            crate::output::line(&format_watch_event(&ev))?;
                        }
                        ipc::BrokerResponse::Error(msg) => {
                            return Err(anyhow::anyhow!("broker watch error: {msg}"));
                        }
                        _ => {}
                    }
                }
            },
            _ = &mut ctrl_c => break,
        }
    }
    let _ = store; // reserved for TUI integration when broker gains a streaming MAP API
    Ok(())
}
