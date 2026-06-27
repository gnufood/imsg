//! Device actor serving: the `Active`-phase select loop, op dispatch, and MNS fan-out.
//!
//! Second impl block for [`Actor`]; the connection lifecycle lives in [`super::inner`].

use anyhow::Result;
use ipc::{BrokerResponse, Reason, WatchEvent};
use map_core::client::MapClient;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, mpsc, oneshot, watch};

use super::{dispatch, Actor, OpOutcome, ServeOutcome};
use crate::runtime::types::DeviceOp;

impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Actor<T> {
    /// Serves [`DeviceOp`]s against the live `client` until idle timeout, all handles dropped
    /// ([`ServeOutcome::Exit`]), or the session dies mid-op ([`ServeOutcome::Dropped`]).
    ///
    /// MNS events are forwarded to subscribers; one-shot ops are dispatched in arrival order.
    pub(in crate::runtime::actor) async fn serve_active(
        &mut self,
        client: &mut MapClient<T>,
    ) -> ServeOutcome {
        loop {
            let sleep = tokio::time::sleep(self.idle);
            tokio::pin!(sleep);
            tokio::select! {
                biased;
                maybe_ev = recv_mns(&mut self.mns_rx) => if let Some(ev) = maybe_ev {
                    let _ = self.watch_tx.send(mns_to_watch(&ev));
                } else {
                    self.mns_rx = None;
                    self.mns_cancel = None;
                },
                maybe_op = self.rx.recv() => {
                    let Some(op) = maybe_op else { return ServeOutcome::Exit };
                    if matches!(self.handle_op(client, op).await, OpOutcome::SessionLost) {
                        return ServeOutcome::Dropped;
                    }
                }
                () = &mut sleep => {
                    tracing::info!("idle timeout — shutting down");
                    return ServeOutcome::Exit;
                }
            }
        }
    }

    /// Dispatches one [`DeviceOp`]. Watch ops adjust the subscriber count; MAP ops run via
    /// [`dispatch`] and finish through [`finish_map`][Self::finish_map].
    async fn handle_op(&mut self, client: &mut MapClient<T>, op: DeviceOp) -> OpOutcome {
        match op {
            DeviceOp::Subscribe { reply } => {
                self.handle_subscribe(reply).await;
                OpOutcome::Continue
            }
            DeviceOp::Unsubscribe => {
                self.watch_count = self.watch_count.saturating_sub(1);
                if self.watch_count == 0 {
                    self.stop_mns();
                }
                OpOutcome::Continue
            }
            DeviceOp::Sync { folder, reply } => {
                let r = dispatch::do_sync(client, &self.store, folder).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::Send { number, message, reply } => {
                let r = dispatch::do_send(client, &self.store, number, message).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::Delete { msg_handle, folder, reply } => {
                let r = dispatch::do_delete(client, &self.store, msg_handle, folder).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::LiveMarkRead { handle, reply } => {
                let r = dispatch::do_live_mark_read(client, handle).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::LiveSend { number, message, reply } => {
                let r = dispatch::do_live_send(client, number, message).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::Backfill { reply } => {
                let r = dispatch::do_backfill(client, &self.store).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::LiveList { folder, unread, from, since, limit, offset, reply } => {
                let r = dispatch::do_live_list(client, folder, unread, from, since, limit, offset)
                    .await;
                Self::finish_map(r, reply)
            }
            DeviceOp::LiveGet { handle, reply } => {
                let r = dispatch::do_live_get(client, handle).await;
                Self::finish_map(r, reply)
            }
            DeviceOp::LiveThreads { reply } => {
                let r = dispatch::do_live_threads(client).await;
                Self::finish_map(r, reply)
            }
        }
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

    /// Registers a watch subscriber, starting MNS on the first one, and returns a fresh receiver.
    async fn handle_subscribe(&mut self, reply: oneshot::Sender<broadcast::Receiver<WatchEvent>>) {
        if self.watch_count == 0 {
            self.start_mns().await;
        }
        self.watch_count = self.watch_count.saturating_add(1);
        let _ = reply.send(self.watch_tx.subscribe());
    }

    /// Starts the MNS listener task; a failure is logged and leaves watch unavailable (local to the
    /// watch subsystem — it does not affect the MAP session state).
    pub(in crate::runtime::actor) async fn start_mns(&mut self) {
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

/// Awaits the next MNS event, or `pending` when MNS is not running.
async fn recv_mns(rx: &mut Option<mpsc::Receiver<session::MnsEvent>>) -> Option<session::MnsEvent> {
    match rx {
        Some(r) => r.recv().await,
        None => std::future::pending().await,
    }
}

/// Flattens an [`session::MnsEvent`] into the wire [`WatchEvent`].
fn mns_to_watch(ev: &session::MnsEvent) -> WatchEvent {
    WatchEvent {
        event_type: ev.event_type().to_string(),
        handle: ev.handle().map(str::to_owned),
        folder: ev.folder().map(str::to_owned),
        old_folder: ev.old_folder().map(str::to_owned),
        msg_type: ev.msg_type().map(str::to_owned),
        datetime: ev.datetime().map(str::to_owned),
    }
}
