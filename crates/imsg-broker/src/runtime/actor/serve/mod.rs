//! Device actor serving: the `Active`-phase select loop, op dispatch, and MNS event handling
//! (store write + subscriber fan-out).
//!
//! Second impl block for [`Actor`]; the connection lifecycle lives in [`super::inner`], PBAP op
//! dispatch lives in [`pbap`], and MNS event handling lives in [`mns`].

mod mns;
mod pbap;

use std::time::Duration;

use anyhow::Result;
use ipc::{BrokerResponse, Reason, WatchEvent};
use map_core::client::MapClient;
use mns::{on_mns_event, recv_mns};
use pbap_core::client::PbapClient;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, mpsc, oneshot, watch};

use super::{dispatch, Actor, OpOutcome, ServeOutcome};
use crate::runtime::types::DeviceOp;

pub(in crate::runtime::actor) use mns::wants_mns;

impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Actor<T> {
    /// Serves [`DeviceOp`]s against the live `client` until idle timeout, all handles dropped
    /// ([`ServeOutcome::Exit`]), or the session dies mid-op ([`ServeOutcome::Dropped`]).
    ///
    /// MNS events are written to the store, then forwarded to `Watch` subscribers, unconditionally
    /// — not gated on a subscriber being attached (the daemon's steady state has none). One-shot
    /// ops are dispatched in arrival order. `self.idle == None` (daemon's persistent mode) disables
    /// the timeout branch entirely.
    pub(in crate::runtime::actor) async fn serve_active(
        &mut self,
        client: &mut MapClient<T>,
        pbap: &mut Option<PbapClient<T>>,
    ) -> ServeOutcome {
        loop {
            tokio::select! {
                biased;
                maybe_ev = recv_mns(&mut self.mns_rx) => if let Some(ev) = maybe_ev {
                    let outcome = on_mns_event(&ev, client, &self.store, &self.watch_tx).await;
                    if matches!(outcome, OpOutcome::SessionLost) {
                        return ServeOutcome::Dropped;
                    }
                } else {
                    self.mns_rx = None;
                    self.mns_cancel = None;
                },
                maybe_op = self.rx.recv() => {
                    let Some(op) = maybe_op else { return ServeOutcome::Exit };
                    if matches!(self.handle_op(client, pbap, op).await, OpOutcome::SessionLost) {
                        return ServeOutcome::Dropped;
                    }
                }
                () = idle_sleep(self.idle) => {
                    tracing::info!("idle timeout — shutting down");
                    return ServeOutcome::Exit;
                }
            }
        }
    }

    /// Dispatches one [`DeviceOp`]. Watch ops adjust the subscriber count; MAP ops run via
    /// [`dispatch`] and finish through [`finish_map`][Self::finish_map]; PBAP ops (contacts sync
    /// and live browsing) delegate to [`handle_pbap_op`][Self::handle_pbap_op].
    async fn handle_op(
        &mut self,
        client: &mut MapClient<T>,
        pbap: &mut Option<PbapClient<T>>,
        op: DeviceOp,
    ) -> OpOutcome {
        match op {
            DeviceOp::Subscribe { reply } => {
                self.handle_subscribe(reply).await;
                OpOutcome::Continue
            }
            DeviceOp::Unsubscribe => self.handle_unsubscribe(),
            DeviceOp::Sync { folder, reply } => {
                Self::finish_map(dispatch::do_sync(client, &self.store, folder).await, reply)
            }
            DeviceOp::Send { number, message, reply } => Self::finish_map(
                dispatch::do_send(client, &self.store, number, message).await,
                reply,
            ),
            DeviceOp::Delete { msg_handle, folder, reply } => Self::finish_map(
                dispatch::do_delete(client, &self.store, msg_handle, folder, &self.watch_tx).await,
                reply,
            ),
            DeviceOp::LiveMarkRead { handle, reply } => {
                Self::finish_map(dispatch::do_live_mark_read(client, handle).await, reply)
            }
            DeviceOp::LiveSend { number, message, reply } => {
                Self::finish_map(dispatch::do_live_send(client, number, message).await, reply)
            }
            DeviceOp::Backfill { reply } => {
                Self::finish_map(dispatch::do_backfill(client, &self.store).await, reply)
            }
            DeviceOp::LiveList { folder, unread, from, since, limit, offset, reply } => {
                Self::finish_map(
                    dispatch::do_live_list(client, folder, unread, from, since, limit, offset)
                        .await,
                    reply,
                )
            }
            DeviceOp::LiveGet { handle, reply } => {
                Self::finish_map(dispatch::do_live_get(client, handle).await, reply)
            }
            DeviceOp::LiveThreads { reply } => {
                Self::finish_map(dispatch::do_live_threads(client).await, reply)
            }
            op @ (DeviceOp::SyncContacts { .. }
            | DeviceOp::ListContacts { .. }
            | DeviceOp::GetContact { .. }
            | DeviceOp::LookupContact { .. }
            | DeviceOp::PullAllContacts { .. }) => self.handle_pbap_op(pbap, op).await,
        }
    }

    /// Drops one `Watch` subscriber, stopping MNS once none remain.
    fn handle_unsubscribe(&mut self) -> OpOutcome {
        self.watch_count = self.watch_count.saturating_sub(1);
        if !wants_mns(self.watch_count, self.idle) {
            self.stop_mns();
        }
        OpOutcome::Continue
    }

    /// Replies to a MAP op and reports whether the session survived.
    ///
    /// A fatal transport error (dead session) becomes [`Reason::DeviceUnreachable`] and signals
    /// [`OpOutcome::SessionLost`] so the actor reconnects; the op is never auto-replayed. A
    /// non-fatal error becomes [`Reason::OperationFailed`] and serving continues.
    fn finish_map(
        result: Result<BrokerResponse>,
        reply: oneshot::Sender<BrokerResponse>,
    ) -> OpOutcome {
        match result {
            Ok(resp) => {
                let _ = reply.send(resp);
                OpOutcome::Continue
            }
            Err(e) if session::outbox::is_fatal_anyhow(&e) => {
                tracing::warn!("session lost during op: {e:#}");
                let _ = reply.send(BrokerResponse::Failed(Reason::DeviceUnreachable));
                OpOutcome::SessionLost
            }
            Err(e) => {
                let _ =
                    reply.send(BrokerResponse::Failed(Reason::OperationFailed(format!("{e:#}"))));
                OpOutcome::Continue
            }
        }
    }

    /// Registers a watch subscriber, starting MNS if it isn't already running (e.g. persistent
    /// mode already started it), and returns a fresh receiver.
    async fn handle_subscribe(&mut self, reply: oneshot::Sender<broadcast::Receiver<WatchEvent>>) {
        if self.mns_rx.is_none() {
            self.start_mns().await;
        }
        self.watch_count = self.watch_count.saturating_add(1);
        let _ = reply.send(self.watch_tx.subscribe());
    }

    /// Starts the MNS listener task, unless one is already running. A failure is logged and
    /// leaves watch unavailable (local to the watch subsystem — it does not affect the MAP
    /// session state).
    ///
    /// Idempotent so callers can call it unconditionally: the MNS profile must be registered
    /// with `BlueZ` *before* the MAP session enables notifications (the phone connects back to
    /// it as soon as it does), so it is started once up front and left running across MAP
    /// reconnects rather than torn down and re-registered every time.
    pub(in crate::runtime::actor) async fn start_mns(&mut self) {
        if self.mns_rx.is_some() {
            return;
        }
        match transport::rfcomm::listen_mns().await {
            Ok(listener) => {
                let (cancel_tx, cancel_rx) = watch::channel(false);
                let (ev_tx, ev_rx) = mpsc::channel(32);
                tokio::spawn(async move {
                    session::mns::run_mns_session(listener, ev_tx, cancel_rx).await;
                });
                self.mns_rx = Some(ev_rx);
                self.mns_cancel = Some(cancel_tx);
            }
            Err(e) => tracing::warn!("MNS listener failed to start: {e}"),
        }
    }

    /// Cancels the MNS listener task, if running.
    pub(in crate::runtime::actor) fn stop_mns(&mut self) {
        if let Some(tx) = self.mns_cancel.take() {
            let _ = tx.send(true);
        }
        self.mns_rx = None;
    }
}

/// Awaits the idle timeout, or `pending` forever when `idle` is `None` (daemon's persistent mode).
async fn idle_sleep(idle: Option<Duration>) {
    match idle {
        Some(d) => tokio::time::sleep(d).await,
        None => std::future::pending().await,
    }
}
