//! Device actor connection lifecycle: bounded connect/retry, reconnect, and state publication.
//!
//! [`Actor::run`] owns the MAP session: it establishes the client via the [`Connector`] with
//! bounded retry, publishes [`ConnState`], serves [`DeviceOp`]s while `Active`, and on a recoverable
//! drop reconnects while subscribers remain or in persistent (daemon) mode (otherwise exits so the
//! CLI respawns lazily). A permanent error or an exhausted budget transitions to terminal
//! [`ConnState::Failed`]. The serving half of the impl lives in [`super::serve`].

use ipc::{BrokerResponse, Reason};
use map_core::client::MapClient;
use session::Disposition;
use tokio::io::{AsyncRead, AsyncWrite};

use super::serve::wants_mns;
use super::{dispatch, Actor, ServeOutcome};
use crate::runtime::types::{ConnState, DeviceOp};

impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Actor<T> {
    /// Runs the connect → serve → reconnect lifecycle until the broker exits.
    ///
    /// Each iteration establishes the session (publishing `Active` and draining the outbox),
    /// restarts MNS if subscribers remain, then serves until idle/shutdown (exit) or a session
    /// drop. A drop with subscribers or in persistent mode reconnects (`Reconnecting`); otherwise
    /// exits so the CLI respawns lazily. A failed connect goes terminal `Failed`. Fires the
    /// shutdown signal on exit.
    pub(in crate::runtime::actor) async fn run(mut self) {
        loop {
            let client = match self.try_connect().await {
                Ok(c) => c,
                Err(reason) => {
                    let _ = self.state_tx.send(ConnState::Failed(reason.clone()));
                    self.fail_pending(&reason);
                    break;
                }
            };
            if !self.run_session(client).await {
                break;
            }
        }
        let _ = self.shutdown_tx.send(true);
    }

    /// Serves one connected session: publishes `Active`, drains the outbox, restarts MNS for any
    /// existing subscribers or persistent (daemon) mode, then serves until the session ends.
    ///
    /// Returns `true` to reconnect (a recoverable drop while subscribers remain or in persistent
    /// mode) or `false` to exit the lifecycle (idle/shutdown, or a drop with no subscribers in
    /// ephemeral mode — the CLI respawns lazily).
    async fn run_session(&mut self, mut client: MapClient<T>) -> bool {
        let _ = self.state_tx.send(ConnState::Active);
        let now = session::util::now_ms();
        if let Err(e) = session::outbox::drain_outbox(&mut client, &self.store, now).await {
            tracing::warn!("initial outbox drain failed: {e}");
        }
        if wants_mns(self.watch_count, self.idle) {
            self.start_mns().await;
        }
        let outcome = self.serve_active(&mut client).await;
        self.stop_mns();
        match outcome {
            ServeOutcome::Exit => {
                if let Err(e) = client.disconnect().await {
                    tracing::warn!("MAP disconnect on shutdown: {e}");
                }
                false
            }
            // Demand-gate: no subscribers and not persistent mode means no one needs the link —
            // exit, respawn lazily. Persistent mode (`idle: None`) always reconnects.
            ServeOutcome::Dropped if !wants_mns(self.watch_count, self.idle) => {
                self.fail_pending(&Reason::DeviceUnreachable);
                false
            }
            ServeOutcome::Dropped => {
                let _ = self.state_tx.send(ConnState::Reconnecting);
                true
            }
        }
    }

    /// Establishes the session, within the wall-clock startup budget if the policy has one.
    ///
    /// `startup_budget: None` (persistent/daemon mode) applies no deadline — only the attempt
    /// budget bounds it, and daemon policies set that to effectively unbounded too.
    ///
    /// Returns the live client, or a terminal [`Reason`] when the budget elapses, the attempt
    /// budget is exhausted, or a permanent error occurs.
    async fn try_connect(&mut self) -> Result<MapClient<T>, Reason> {
        match self.policy.startup_budget {
            Some(budget) => match tokio::time::timeout(budget, self.connect_with_retry()).await {
                Ok(result) => result,
                Err(_elapsed) => Err(Reason::DeviceUnreachable),
            },
            None => self.connect_with_retry().await,
        }
    }

    /// Calls the connector with doubling backoff, retrying transient failures up to the attempt
    /// budget and failing fast on permanent ones. Backoff resets each phase (fresh schedule).
    async fn connect_with_retry(&mut self) -> Result<MapClient<T>, Reason> {
        let mut delays = session::retry::backoff(
            self.policy.initial_backoff,
            self.policy.max_backoff,
            self.policy.max_attempts,
        );
        loop {
            let err = match (self.connect)().await {
                Ok(client) => return Ok(client),
                Err(e) => e,
            };
            let reason = dispatch::connect_reason(&err);
            if session::classify(&err) == Disposition::Permanent {
                tracing::error!("MAP connect failed (permanent): {err}");
                return Err(reason);
            }
            let Some(d) = delays.next() else {
                tracing::error!("MAP connect failed; attempts exhausted: {err}");
                return Err(reason);
            };
            tracing::warn!("MAP connect failed (transient): {err}; retrying in {d:?}");
            tokio::time::sleep(d).await;
        }
    }

    /// Replies [`BrokerResponse::Failed`] to every operational op still queued on exit.
    ///
    /// Without this, ops queued behind a request that killed the session would have their reply
    /// channels dropped, surfacing to the CLI as a generic "actor dropped reply" IPC error instead
    /// of the real, action-oriented [`Reason`]. Watch ops carry no `BrokerResponse` reply and are
    /// discarded.
    fn fail_pending(&mut self, reason: &Reason) {
        while let Ok(op) = self.rx.try_recv() {
            let resp = BrokerResponse::Failed(reason.clone());
            match op {
                DeviceOp::Sync { reply, .. }
                | DeviceOp::Send { reply, .. }
                | DeviceOp::Delete { reply, .. }
                | DeviceOp::Backfill { reply }
                | DeviceOp::LiveList { reply, .. }
                | DeviceOp::LiveGet { reply, .. }
                | DeviceOp::LiveThreads { reply }
                | DeviceOp::LiveMarkRead { reply, .. }
                | DeviceOp::LiveSend { reply, .. } => {
                    let _ = reply.send(resp);
                }
                DeviceOp::Subscribe { .. } | DeviceOp::Unsubscribe => {}
            }
        }
    }
}
