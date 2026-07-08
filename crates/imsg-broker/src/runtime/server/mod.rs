//! Abstract-socket lifecycle: bind-or-exit election and concurrent accept loop.
//!
//! Each accepted connection is dispatched to its own task (see [`super::handler`]) so `Watch`
//! subscribers never block one-shot commands. The idle timeout and connection lifecycle live in
//! the device actor; [`serve_loop`] runs until it receives the actor's shutdown signal.
//!
//! The kernel releases the abstract name on process exit (including crash or SIGKILL), so there is
//! no socket file to clean up on any code path.

use std::time::Duration;

use anyhow::{Context, Result};
use bluer::rfcomm::Stream;
use config::Config;
use interprocess::local_socket::{
    tokio::prelude::*, tokio::Listener as IpcListener, ListenerOptions,
};
use store::Store;

use super::handler::handle_connection;
use crate::runtime::types::{ActorHandles, ConnectPolicy, Connector};

/// Binds the abstract socket for `addr`.
///
/// `EADDRINUSE` on an abstract name is a kernel-atomic single-instance election: only one
/// process wins the bind. No file, no TOCTOU, no cleanup. When `allow_silent_exit` is `true`
/// (the ephemeral one-shot broker, auto-spawned redundantly by racing CLI commands) a losing
/// process exits 0 immediately — something is already there to serve. When `false` (the
/// persistent daemon's `--foreground` entry point, including under a service supervisor)
/// losing the race returns an error instead: silently exiting 0 here would be indistinguishable
/// from a successful start to `RestartPolicy::OnFailure`, masking a lost startup race as success.
///
/// # Errors
///
/// Returns an error if `allow_silent_exit` is `false` and the socket is already held, or for
/// bind failures other than `EADDRINUSE` (e.g. kernel resource limits).
pub(in crate::runtime) fn bind_or_exit(addr: &str, allow_silent_exit: bool) -> Result<IpcListener> {
    let name = config::broker_abstract_name(addr).context("constructing abstract socket name")?;
    match ListenerOptions::new().name(name).create_tokio() {
        Ok(l) => Ok(l),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse && allow_silent_exit => {
            tracing::info!("broker already running for {addr} — exiting");
            std::process::exit(0);
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Err(anyhow::anyhow!(
            "another broker instance already holds the socket for {addr} — refusing to start \
             silently; check whether a daemon is already running (`imsg daemon status`)"
        )),
        Err(e) => Err(anyhow::Error::from(e).context("binding abstract broker socket")),
    }
}

/// Spawns the device actor and runs the accept loop until idle timeout or terminal MAP failure.
///
/// Builds the connect policy and readiness deadline from `cfg`, then serves IPC connections. The
/// abstract socket is released by the kernel when this returns and `listener` is dropped.
///
/// # Errors
///
/// Returns an error if `listener.accept()` fails fatally.
pub(in crate::runtime) async fn serve(
    cfg: Config,
    device: String,
    addr: bluer::Address,
    channel: u8,
    store: Store,
    listener: IpcListener,
) -> Result<()> {
    let idle = Some(cfg.broker.idle());
    serve_with_idle(cfg, device, addr, channel, store, listener, idle).await
}

/// Persistent-mode counterpart to [`serve`]: idle disabled, and — unlike [`serve`]'s plain
/// [`serve_loop`] — accepts through [`super::shutdown::run`], which converges an IPC `Shutdown`
/// request and SIGTERM/SIGINT into one graceful drain. `serve` deliberately does not get this:
/// the ephemeral one-shot broker's shutdown timing stays exactly as-is, zero risk to existing
/// CLI commands.
///
/// # Errors
///
/// Returns an error if `listener.accept()` fails fatally.
pub(in crate::runtime) async fn serve_daemon(
    cfg: Config,
    device: String,
    addr: bluer::Address,
    channel: u8,
    store: Store,
    listener: IpcListener,
) -> Result<()> {
    let policy = build_daemon_policy(&cfg);
    let readiness_wait = cfg.broker.readiness_wait();
    let connector = make_connector(addr, channel, cfg.broker.bt_connected());
    super::shutdown::run(connector, store, policy, listener, device, readiness_wait).await
}

/// Shared by [`serve`] and [`serve_daemon`]; `idle` is their only difference.
async fn serve_with_idle(
    cfg: Config,
    device: String,
    addr: bluer::Address,
    channel: u8,
    store: Store,
    listener: IpcListener,
    idle: Option<Duration>,
) -> Result<()> {
    let policy = build_policy(&cfg);
    let readiness_wait = cfg.broker.readiness_wait();
    let connector = make_connector(addr, channel, cfg.broker.bt_connected());
    serve_actor(connector, store, idle, policy, &listener, device, readiness_wait).await
}

/// Builds the connect-retry policy for one-shot (`serve`) mode: bounded attempts within a
/// wall-clock budget, so a CLI command fails fast and reports a clear error when the device
/// isn't reachable, rather than hanging.
pub(in crate::runtime) const fn build_policy(cfg: &Config) -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: cfg.broker.initial_backoff(),
        max_backoff: cfg.broker.max_backoff(),
        max_attempts: cfg.broker.connect_max_attempts,
        startup_budget: Some(cfg.broker.startup_budget()),
    }
}

/// Builds the connect-retry policy for persistent (`serve_daemon`) mode: unbounded attempts,
/// no wall-clock deadline. A daemon started before the phone is in Bluetooth range should keep
/// retrying (capped backoff, same schedule as one-shot mode) until it connects, not give up.
pub(in crate::runtime) const fn build_daemon_policy(cfg: &Config) -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: cfg.broker.initial_backoff(),
        max_backoff: cfg.broker.max_backoff(),
        max_attempts: u32::MAX,
        startup_budget: None,
    }
}

/// Connector-generic core of [`serve`]/[`serve_daemon`], split out so the idle-wiring behavior
/// is testable against a fake in-memory connector instead of a real MAP session.
async fn serve_actor<T>(
    connector: Connector<T>,
    store: Store,
    idle: Option<Duration>,
    policy: ConnectPolicy,
    listener: &IpcListener,
    device: String,
    readiness_wait: Duration,
) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let handles = super::actor::spawn(connector, store, idle, policy);
    serve_loop(handles, listener, device, readiness_wait).await
}

/// Builds the production connector: every call establishes a fresh RFCOMM/OBEX MAP session to
/// `addr`:`channel`, gating on `BT_CONNECTED` up to `bt_gate`. Defined here so the transport-specific
/// stream type stays out of the actor.
fn make_connector(addr: bluer::Address, channel: u8, bt_gate: Duration) -> Connector<Stream> {
    Box::new(move || Box::pin(session::lifecycle::connect_map(addr, channel, bt_gate)))
}

/// Accepts connections until the actor signals shutdown. Each connection runs in its own task with
/// a clone of the op handle and the connection-state watch.
async fn serve_loop(
    handles: ActorHandles,
    listener: &IpcListener,
    device: String,
    readiness_wait: Duration,
) -> Result<()> {
    let ActorHandles { handle, state, mut shutdown } = handles;
    loop {
        let stream = tokio::select! {
            biased;
            result = shutdown.changed() => {
                // Sender dropped (actor gone) or a reason published — shut down either way.
                // The ephemeral broker doesn't distinguish why; only the daemon path does.
                if result.is_err() || shutdown.borrow_and_update().is_some() {
                    return Ok(());
                }
                continue;
            }
            result = listener.accept() => result.context("accept error")?,
        };
        let h = handle.clone();
        let st = state.clone();
        let dev = device.clone();
        tokio::spawn(async move {
            // `None`: the ephemeral broker has no shutdown coordinator to cancel.
            if let Err(e) = handle_connection(stream, &h, st, dev, readiness_wait, None).await {
                tracing::warn!("connection error: {e}");
            }
        });
    }
}

#[cfg(test)]
mod tests;
