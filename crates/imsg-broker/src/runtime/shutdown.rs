//! Graceful shutdown coordinator for persistent (daemon) mode.
//!
//! An IPC `Shutdown` request and SIGTERM/SIGINT both cancel the same [`CancellationToken`],
//! converging on one drain sequence: stop accepting connections, let in-flight ones finish
//! (bounded), drop the actor handle so it disconnects and exits, confirm that exit (bounded).
//! Not used by the ephemeral one-shot broker — see [`super::server::serve_daemon`].

use std::time::Duration;

use anyhow::{Context, Result};
use interprocess::local_socket::{tokio::prelude::*, tokio::Listener as IpcListener};
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use super::handler::handle_connection;
use super::types::{ActorHandles, ConnectPolicy, Connector};

/// Bound on how long shutdown waits for in-flight connections to finish and for the actor to
/// confirm it disconnected, each. Not yet configurable.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Spawns the device actor with idle disabled and serves until an external stop is requested.
///
/// # Errors
///
/// Returns an error if `listener.accept()` fails fatally.
pub(in crate::runtime) async fn run<T>(
    connector: Connector<T>,
    store: Store,
    policy: ConnectPolicy,
    listener: &IpcListener,
    device: String,
    readiness_wait: Duration,
) -> Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let token = CancellationToken::new();
    install_signal_handlers(token.clone());
    let handles = super::actor::spawn(connector, store, None, policy);
    accept_and_drain(handles, listener, device, readiness_wait, token).await
}

/// Accepts connections until the actor exits on its own or `token` is cancelled, then drains.
///
/// Per-connection tasks run through a [`TaskTracker`] (not bare `tokio::spawn`, unlike the
/// ephemeral broker's [`super::server`] accept loop) so shutdown can wait for them.
async fn accept_and_drain(
    handles: ActorHandles,
    listener: &IpcListener,
    device: String,
    readiness_wait: Duration,
    token: CancellationToken,
) -> Result<()> {
    let ActorHandles { handle, state, mut shutdown } = handles;
    let tasks = TaskTracker::new();
    loop {
        let stream = tokio::select! {
            biased;
            result = shutdown.changed() => {
                if result.is_err() || *shutdown.borrow_and_update() {
                    break;
                }
                continue;
            }
            () = token.cancelled() => break,
            result = listener.accept() => result.context("accept error")?,
        };
        let h = handle.clone();
        let st = state.clone();
        let dev = device.clone();
        let tok = token.clone();
        tasks.spawn(async move {
            if let Err(e) = handle_connection(stream, &h, st, dev, readiness_wait, Some(&tok)).await
            {
                tracing::warn!("connection error: {e}");
            }
        });
    }

    // Dropping `handle` closes the actor's op channel once every in-flight clone above is also
    // gone (tracked by `tasks`); that's the actor's only lever to notice it should exit.
    drop(handle);
    tasks.close();
    let _ = tokio::time::timeout(DRAIN_TIMEOUT, tasks.wait()).await;
    let _ = tokio::time::timeout(DRAIN_TIMEOUT, shutdown.wait_for(|s| *s)).await;
    Ok(())
}

/// Cancels `token` on SIGTERM or SIGINT. Runs detached — this task's only job is forwarding
/// the signal, so it needs no join handle or tracking.
#[cfg(unix)]
fn install_signal_handlers(token: CancellationToken) {
    tokio::spawn(async move {
        let (mut term, mut int) =
            match (signal(SignalKind::terminate()), signal(SignalKind::interrupt())) {
                (Ok(t), Ok(i)) => (t, i),
                (Err(e), _) | (_, Err(e)) => {
                    tracing::warn!("failed to install signal handlers: {e}");
                    return;
                }
            };
        tokio::select! {
            _ = term.recv() => tracing::info!("received SIGTERM — shutting down"),
            _ = int.recv() => tracing::info!("received SIGINT — shutting down"),
        }
        token.cancel();
    });
}

#[cfg(test)]
mod tests;
