//! Graceful shutdown coordinator for persistent (daemon) mode.
//!
//! An IPC `Shutdown` request and SIGTERM/SIGINT both cancel the same [`CancellationToken`],
//! converging on one drain sequence: stop accepting connections and drop the listener immediately
//! (so a connection racing the stop gets a clean refusal/reset right away instead of queuing into
//! a backlog nobody will ever `accept()` again), let already-accepted connections finish (bounded),
//! drop the actor handle so it disconnects and exits, confirm that exit (bounded). Not used by the
//! ephemeral one-shot broker — see [`super::server::serve_daemon`].

use std::time::Duration;

use anyhow::{Context, Result};
use interprocess::local_socket::{tokio::prelude::*, tokio::Listener as IpcListener};
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use super::handler::handle_connection;
use super::types::{
    ActorHandles, ConnState, ConnectPolicy, Connectors, DeviceHandle, TerminalReason,
};

/// Bound on how long shutdown waits for in-flight connections to finish and for the actor to
/// confirm it disconnected, each. Not yet configurable.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Spawns the device actor with idle disabled and serves until an external stop is requested.
///
/// # Errors
///
/// Returns an error if `listener.accept()` fails fatally.
pub(in crate::runtime) async fn run<T>(
    connectors: Connectors<T>,
    store: Store,
    policy: ConnectPolicy,
    listener: IpcListener,
    device: String,
    readiness_wait: Duration,
) -> Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let token = CancellationToken::new();
    #[cfg(unix)]
    install_signal_handlers(token.clone());
    #[cfg(not(unix))]
    tracing::warn!(
        "no SIGTERM/SIGINT handling on this platform — only an IPC `Shutdown` request will stop \
         the daemon"
    );
    let handles =
        super::actor::spawn(connectors.map, connectors.pbap, connectors.link, store, None, policy);
    accept_and_drain(handles, listener, device, readiness_wait, token).await
}

/// Accepts connections until the actor exits on its own or `token` is cancelled, then drains.
///
/// Per-connection tasks run through a [`TaskTracker`] (not bare `tokio::spawn`, unlike the
/// ephemeral broker's [`super::server`] accept loop) so shutdown can wait for them.
///
/// `listener` is owned (not borrowed) so it can be dropped the instant this stops accepting,
/// rather than staying bound for the whole bounded drain — otherwise a client that connects
/// during that window queues into a backlog this loop will never `accept()` again, and only
/// discovers that via a raw connection reset once the listener eventually drops.
///
/// # Errors
///
/// Returns an error if `listener.accept()` fails fatally, or if the actor's connect phase
/// exhausted its retry budget or hit a permanent MAP error ([`TerminalReason::PermanentFailure`])
/// rather than exiting via a requested stop — so the process exits non-zero on a genuine
/// unrecoverable failure instead of looking identical to a clean `imsg daemon stop`.
async fn accept_and_drain(
    handles: ActorHandles,
    listener: IpcListener,
    device: String,
    readiness_wait: Duration,
    token: CancellationToken,
) -> Result<()> {
    let ActorHandles { handle, state, mut shutdown } = handles;
    let tasks = TaskTracker::new();
    let cfg = AcceptConfig {
        listener: &listener,
        device: &device,
        readiness_wait,
        token: &token,
        tasks: &tasks,
    };
    run_accept_loop(&handle, &state, &mut shutdown, cfg).await?;

    // Drop the listener now, not when this function returns — a connection that manages to
    // connect() into the kernel backlog after we stop accepting but before this drop would
    // otherwise sit unread until the listener eventually goes away, then see a raw connection
    // reset instead of a clean, immediate refusal.
    drop(listener);

    // Dropping `handle` closes the actor's op channel once every in-flight clone above is also
    // gone (tracked by `tasks`); that's the actor's only lever to notice it should exit.
    drop(handle);
    tasks.close();
    let _ = tokio::time::timeout(DRAIN_TIMEOUT, tasks.wait()).await;
    let _ = tokio::time::timeout(DRAIN_TIMEOUT, shutdown.wait_for(Option::is_some)).await;
    let reason = shutdown.borrow().clone();
    match reason {
        Some(TerminalReason::PermanentFailure(reason)) => {
            Err(anyhow::anyhow!("daemon stopped: unrecoverable MAP failure: {reason:?}"))
        }
        _ => Ok(()),
    }
}

/// Immutable per-connection config shared by every task [`run_accept_loop`] spawns — bundled so
/// the loop itself stays under the argument-count ceiling.
struct AcceptConfig<'a> {
    listener: &'a IpcListener,
    device: &'a str,
    readiness_wait: Duration,
    token: &'a CancellationToken,
    tasks: &'a TaskTracker,
}

/// Accepts connections into `cfg.tasks` until `cfg.token` cancels or the actor's `shutdown`
/// watch fires. Split out of [`accept_and_drain`] to keep it under the size ceiling.
///
/// # Errors
///
/// Returns an error if `cfg.listener.accept()` fails fatally.
async fn run_accept_loop(
    handle: &DeviceHandle,
    state: &watch::Receiver<ConnState>,
    shutdown: &mut watch::Receiver<Option<TerminalReason>>,
    cfg: AcceptConfig<'_>,
) -> Result<()> {
    loop {
        let stream = tokio::select! {
            biased;
            result = shutdown.changed() => {
                if result.is_err() || shutdown.borrow_and_update().is_some() {
                    return Ok(());
                }
                continue;
            }
            () = cfg.token.cancelled() => return Ok(()),
            result = cfg.listener.accept() => result.context("accept error")?,
        };
        let h = handle.clone();
        let st = state.clone();
        let dev = cfg.device.to_owned();
        let tok = cfg.token.clone();
        let readiness_wait = cfg.readiness_wait;
        cfg.tasks.spawn(async move {
            if let Err(e) = handle_connection(stream, &h, st, dev, readiness_wait, Some(&tok)).await
            {
                tracing::warn!("connection error: {e}");
            }
        });
    }
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
