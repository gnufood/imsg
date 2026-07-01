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

/// Binds the abstract socket for `addr`, or exits 0 if another broker already holds it.
///
/// `EADDRINUSE` on an abstract name is a kernel-atomic single-instance election: only one
/// process wins the bind; all others exit immediately. No file, no TOCTOU, no cleanup.
///
/// # Errors
///
/// Returns an error for bind failures other than `EADDRINUSE` (e.g. kernel resource limits).
pub(in crate::runtime) fn bind_or_exit(addr: &str) -> Result<IpcListener> {
    let name = config::broker_abstract_name(addr).context("constructing abstract socket name")?;
    match ListenerOptions::new().name(name).create_tokio() {
        Ok(l) => Ok(l),
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            tracing::info!("broker already running for {addr} — exiting");
            std::process::exit(0);
        }
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
    let policy = ConnectPolicy {
        initial_backoff: cfg.broker.initial_backoff(),
        max_backoff: cfg.broker.max_backoff(),
        max_attempts: cfg.broker.connect_max_attempts,
        startup_budget: cfg.broker.startup_budget(),
    };
    let readiness_wait = cfg.broker.readiness_wait();
    let connector = make_connector(addr, channel, cfg.broker.bt_connected());
    let handles = super::actor::spawn(connector, store, Some(cfg.broker.idle()), policy);
    serve_loop(handles, &listener, device, readiness_wait).await
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
                // Sender dropped (actor gone) or value set true — shut down either way.
                if result.is_err() || *shutdown.borrow_and_update() {
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
            if let Err(e) = handle_connection(stream, &h, st, dev, readiness_wait).await {
                tracing::warn!("connection error: {e}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName as _};

    /// The kernel produces `EADDRINUSE` when the abstract name is already bound.
    ///
    /// `bind_or_exit` maps this to `process::exit(0)`. This test verifies the invariant it
    /// relies on without calling `exit` (which would kill the test process).
    #[tokio::test]
    async fn abstract_name_election_is_atomic() -> anyhow::Result<()> {
        let n1 = "imsg/broker/FE:ED:DE:AD:00:01".to_ns_name::<GenericNamespaced>()?;
        let n2 = "imsg/broker/FE:ED:DE:AD:00:01".to_ns_name::<GenericNamespaced>()?;
        let _l1 = ListenerOptions::new().name(n1).create_tokio()?;
        let Err(err) = ListenerOptions::new().name(n2).create_tokio() else {
            return Err(anyhow::anyhow!(
                "second bind to a held abstract name unexpectedly succeeded"
            ));
        };
        assert_eq!(err.kind(), std::io::ErrorKind::AddrInUse);
        Ok(())
    }

    /// The abstract name is released the instant the listener is dropped.
    ///
    /// This is the anti-regression for the entire stale-socket bug class: with filesystem
    /// sockets a crash leaves an inode behind; with abstract sockets the kernel cleans up
    /// atomically on any exit.
    #[tokio::test]
    async fn abstract_name_released_on_listener_drop() -> anyhow::Result<()> {
        let n1 = "imsg/broker/FE:ED:DE:AD:00:02".to_ns_name::<GenericNamespaced>()?;
        let n2 = "imsg/broker/FE:ED:DE:AD:00:02".to_ns_name::<GenericNamespaced>()?;
        let l = ListenerOptions::new().name(n1).create_tokio()?;
        drop(l);
        // Would be EADDRINUSE if the abstract name leaked.
        ListenerOptions::new().name(n2).create_tokio()?;
        Ok(())
    }
}
