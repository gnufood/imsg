//! Device actor: owns the `MapClient` and (lazily) a `PbapClient`, publishing the MAP session's
//! connection state over a `watch` channel. All MAP operations are serialised through the actor;
//! concurrent CLI connections queue behind the active request via bounded `mpsc`. `Status` is
//! served by connection tasks straight from the state watch, so it never queues behind the device.
//! PBAP ops share the same held-session, serve-many-ops shape as MAP, but are their own fault
//! domain: a lost PBAP session reconnects independently and never marks the MAP session lost (or
//! vice versa) — see `ensure_pbap` in [`inner`].
//!
//! The lifecycle (connect/retry/reconnect, for both sessions) lives in [`inner`]; the serve loop
//! in [`serve`]; the MAP operation helpers in [`dispatch`]. This module owns the actor's private
//! state and the [`spawn`] entry point.

mod dispatch;
mod dto;
mod inner;
mod serve;

use std::time::Duration;

use ipc::WatchEvent;
use store::Store;
use tokio::sync::{broadcast, mpsc, watch};

use super::types::{
    ActorHandles, ConnState, ConnectPolicy, Connector, DeviceHandle, DeviceOp, LinkEvents,
    LinkWatcher, PbapConnector, TerminalReason,
};

/// Owns the connection lifecycle and serves [`DeviceOp`]s from connection tasks.
///
/// The held PBAP session lives as a local in `run` (see [`inner`]), not a field here — same
/// reasoning as the MAP `client`: it is owned by the connect/serve loop, not the actor's own
/// state, so it can be threaded down to `handle_op` by `&mut` without conflicting with `&self`.
struct Actor<T> {
    rx: mpsc::Receiver<DeviceOp>,
    connect: Connector<T>,
    pbap_connect: PbapConnector<T>,
    store: Store,
    idle: Option<Duration>,
    policy: ConnectPolicy,
    watch_tx: broadcast::Sender<WatchEvent>,
    watch_count: u32,
    mns_rx: Option<mpsc::Receiver<session::MnsEvent>>,
    mns_cancel: Option<watch::Sender<bool>>,
    link_watch: LinkWatcher,
    link: Option<LinkEvents>,
    /// Whether a link subscription has been made yet — gates the resubscribe backoff so the
    /// first subscription is immediate.
    link_subscribed: bool,
    state_tx: watch::Sender<ConnState>,
    shutdown_tx: watch::Sender<Option<TerminalReason>>,
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
    pbap_connector: PbapConnector<T>,
    link_watch: LinkWatcher,
    store: Store,
    idle: Option<Duration>,
    policy: ConnectPolicy,
) -> ActorHandles
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (op_tx, op_rx) = mpsc::channel(16);
    let (watch_tx, _initial_rx) = broadcast::channel(64);
    let (state_tx, state_rx) = watch::channel(ConnState::Connecting);
    let (shutdown_tx, shutdown_rx) = watch::channel(None);

    let actor = Actor {
        rx: op_rx,
        connect: connector,
        pbap_connect: pbap_connector,
        link_watch,
        link: None,
        link_subscribed: false,
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
mod tests;
