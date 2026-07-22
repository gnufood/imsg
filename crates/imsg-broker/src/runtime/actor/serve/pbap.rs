//! PBAP op dispatch — split out of `super` to keep that module under the size ceiling. Kept
//! separate at the `handle_op` level too (one delegating match arm, not five) since every PBAP
//! op shares [`run_pbap_op`]'s connect/reuse/fault-isolation behavior, distinct from the MAP
//! ops' [`finish_map`][super::Actor::finish_map]-based handling.

use anyhow::Result;
use futures::future::BoxFuture;
use ipc::{BrokerResponse, Reason};
use pbap_core::client::PbapClient;
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};

use super::super::{dispatch, Actor, OpOutcome};
use crate::runtime::types::DeviceOp;

impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Actor<T> {
    /// Dispatches one PBAP [`DeviceOp`] (`SyncContacts`/`ListContacts`/`GetContact`/
    /// `LookupContact`/`PullAllContacts`) against the held session. Always returns
    /// [`OpOutcome::Continue`] — PBAP failures never signal [`OpOutcome::SessionLost`], since
    /// that would incorrectly reconnect the unrelated MAP session.
    ///
    /// # Panics
    ///
    /// Panics if called with a non-PBAP `op` — callers must only route the PBAP variants here.
    pub(in crate::runtime::actor) async fn handle_pbap_op(
        &mut self,
        pbap: &mut Option<PbapClient<T>>,
        op: DeviceOp,
    ) -> OpOutcome {
        match op {
            DeviceOp::SyncContacts { reply } => {
                let resp = self
                    .run_pbap_op(pbap, |client, store| {
                        Box::pin(dispatch::do_sync_contacts(client, store))
                    })
                    .await;
                let _ = reply.send(resp);
            }
            DeviceOp::ListContacts { path, limit, offset, reply } => {
                let resp = self
                    .run_pbap_op(pbap, |client, _store| {
                        Box::pin(dispatch::do_contacts_list(client, path, limit, offset))
                    })
                    .await;
                let _ = reply.send(resp);
            }
            DeviceOp::GetContact { path, handle, reply } => {
                let resp = self
                    .run_pbap_op(pbap, |client, _store| {
                        Box::pin(dispatch::do_contacts_get(client, path, handle))
                    })
                    .await;
                let _ = reply.send(resp);
            }
            DeviceOp::LookupContact { path, number, reply } => {
                let resp = self
                    .run_pbap_op(pbap, |client, _store| {
                        Box::pin(dispatch::do_contacts_lookup(client, path, number))
                    })
                    .await;
                let _ = reply.send(resp);
            }
            DeviceOp::PullAllContacts { path, limit, offset, reply } => {
                let resp = self
                    .run_pbap_op(pbap, |client, _store| {
                        Box::pin(dispatch::do_contacts_pull_all(client, path, limit, offset))
                    })
                    .await;
                let _ = reply.send(resp);
            }
            _ => unreachable!("handle_pbap_op called with a non-PBAP DeviceOp"),
        }
        OpOutcome::Continue
    }

    /// Runs one PBAP operation against the actor's held session (connecting it first if
    /// needed), converting any failure — connect or op — into `BrokerResponse::Failed`. Never
    /// returns `OpOutcome::SessionLost`: PBAP failures are unrelated to the (unaffected) MAP
    /// session's health. An op failure drops the cached session so the next PBAP op reconnects
    /// rather than reusing a client that may be dead.
    async fn run_pbap_op<F>(&mut self, pbap: &mut Option<PbapClient<T>>, op: F) -> BrokerResponse
    where
        F: for<'a> FnOnce(
            &'a mut PbapClient<T>,
            &'a Store,
        ) -> BoxFuture<'a, Result<BrokerResponse>>,
    {
        if let Err(reason) = self.ensure_pbap(pbap).await {
            return BrokerResponse::Failed(reason);
        }
        // Invariant: `ensure_pbap`'s `Ok(())` always populates `pbap`; the `None` arm is
        // unreachable in practice but handled rather than panicked on.
        let Some(client) = pbap.as_mut() else {
            return BrokerResponse::Failed(Reason::OperationFailed(
                "pbap session missing after connect".to_owned(),
            ));
        };
        match op(client, &self.store).await {
            Ok(resp) => resp,
            Err(e) => {
                *pbap = None;
                BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))
            }
        }
    }
}
