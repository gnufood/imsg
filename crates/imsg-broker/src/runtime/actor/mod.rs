//! Device actor: owns the `MapClient` exclusively and publishes its connection state over a
//! `watch` channel. All MAP operations are serialised through it; concurrent CLI connections queue
//! behind the active request via bounded `mpsc`. `Status` is served by connection tasks straight
//! from the state watch, so it never queues behind the device.
//!
//! The lifecycle (connect/retry/reconnect) lives in [`inner`]; the serve loop in [`serve`]; the MAP
//! operation helpers in [`dispatch`]. This module owns the actor's private state and the [`spawn`]
//! entry point.

mod dispatch;
mod dto;
mod inner;
mod serve;

use std::time::Duration;

use ipc::WatchEvent;
use store::Store;
use tokio::sync::{broadcast, mpsc, watch};

use super::types::{ActorHandles, ConnState, ConnectPolicy, Connector, DeviceHandle, DeviceOp};

/// Owns the connection lifecycle and serves [`DeviceOp`]s from connection tasks.
struct Actor<T> {
    rx: mpsc::Receiver<DeviceOp>,
    connect: Connector<T>,
    store: Store,
    idle: Duration,
    policy: ConnectPolicy,
    watch_tx: broadcast::Sender<WatchEvent>,
    watch_count: u32,
    mns_rx: Option<mpsc::Receiver<session::MnsEvent>>,
    mns_cancel: Option<watch::Sender<bool>>,
    state_tx: watch::Sender<ConnState>,
    shutdown_tx: watch::Sender<bool>,
}

/// Why [`Actor::serve_active`] stopped serving the current session.
enum ServeOutcome {
    /// Idle timeout fired or all handles were dropped — the broker should exit.
    Exit,
    /// The MAP session died — reconnect if subscribers remain, else exit.
    Dropped,
}

/// Outcome of dispatching one [`DeviceOp`] against the live client.
enum OpOutcome {
    /// Keep serving.
    Continue,
    /// The session died mid-operation; leave [`Actor::serve_active`].
    SessionLost,
}

/// Spawns the device actor and returns its handles.
///
/// The actor establishes its MAP session lazily by calling `connector` (with bounded retry per
/// `policy`), so the abstract socket can be bound and served before the slow Bluetooth connection
/// completes. It reconnects on a recoverable drop while subscribers remain, exits after `idle` with
/// no incoming [`DeviceOp`], and goes terminal [`ConnState::Failed`] on a permanent error or an
/// exhausted budget.
pub(in crate::runtime) fn spawn<T>(
    connector: Connector<T>,
    store: Store,
    idle: Duration,
    policy: ConnectPolicy,
) -> ActorHandles
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (op_tx, op_rx) = mpsc::channel(16);
    let (watch_tx, _initial_rx) = broadcast::channel(64);
    let (state_tx, state_rx) = watch::channel(ConnState::Connecting);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let actor = Actor {
        rx: op_rx,
        connect: connector,
        store,
        idle,
        policy,
        watch_tx,
        watch_count: 0,
        mns_rx: None,
        mns_cancel: None,
        state_tx,
        shutdown_tx,
    };
    tokio::spawn(actor.run());

    ActorHandles { handle: DeviceHandle { tx: op_tx }, state: state_rx, shutdown: shutdown_rx }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures::{SinkExt as _, StreamExt as _};
    use secrecy::SecretBox;
    use session::SessionError;
    use tokio::io::duplex;

    const MAP_CONNECT_RSP: &[u8] =
        include_bytes!("../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
    const NOTIF_REG_OK: &[u8] = &[0xA0, 0x00, 0x03];
    const GENERIC_OK: &[u8] = &[0xA0, 0x00, 0x03];

    /// Builds a connector whose every call yields a `MapClient<DuplexStream>` backed by a fresh
    /// minimal fake OBEX server, mirroring the production connector's contract.
    fn fake_connector() -> Connector<tokio::io::DuplexStream> {
        Box::new(|| {
            Box::pin(async {
                let (client_io, server_io) = duplex(4096);
                tokio::spawn(async move {
                    let mut t = obex_core::wrap(server_io);
                    t.send(Bytes::from_static(MAP_CONNECT_RSP)).await.ok();
                    t.next().await; // consume SetNotificationRegistration
                    t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                    while t.next().await.is_some() {
                        t.send(Bytes::from_static(GENERIC_OK)).await.ok();
                    }
                });
                session::lifecycle::establish_map_session(client_io).await
            })
        })
    }

    /// A connector that always fails with a transient transport error.
    fn failing_connector() -> Connector<tokio::io::DuplexStream> {
        Box::new(|| {
            Box::pin(async {
                Err(SessionError::Transport(obex_core::TransportError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "no link",
                ))))
            })
        })
    }

    /// In-memory `Store` (temp-dir `SQLite`) plus the dir guard.
    async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
        let s = Store::open(dir.path().join("test.db"), key).await?;
        Ok((s, dir))
    }

    /// Fast policy for tests: millisecond backoff, two attempts.
    fn test_policy() -> ConnectPolicy {
        ConnectPolicy {
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(2),
            max_attempts: 2,
            startup_budget: Duration::from_secs(5),
        }
    }

    #[tokio::test]
    async fn reaches_active_then_shuts_down_when_handle_dropped() -> anyhow::Result<()> {
        let (store, _dir) = fake_store().await?;
        let h = spawn(fake_connector(), store, Duration::from_secs(60), test_policy());
        let mut state = h.state.clone();
        state.wait_for(|s| matches!(s, ConnState::Active)).await?;
        drop(h.handle);
        let mut shutdown = h.shutdown;
        tokio::time::timeout(Duration::from_secs(3), shutdown.changed()).await??;
        assert!(*shutdown.borrow());
        Ok(())
    }

    #[tokio::test]
    async fn failed_connect_goes_terminal() -> anyhow::Result<()> {
        let (store, _dir) = fake_store().await?;
        let h = spawn(failing_connector(), store, Duration::from_secs(60), test_policy());
        let mut state = h.state.clone();
        state.wait_for(|s| matches!(s, ConnState::Failed(_))).await?;
        Ok(())
    }
}
