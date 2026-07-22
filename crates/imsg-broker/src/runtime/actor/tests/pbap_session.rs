//! Tests for the actor's held PBAP session — reuse across ops, and independent fault isolation
//! from the MAP session. Lifecycle-only (connect/reconnect) tests live in [`super::lifecycle`].

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::*;

/// Builds a counting PBAP connector whose single connection serves two full
/// `sync_contacts` cycles before the fake server task ends — used to prove a second
/// `SyncContacts` op reuses the held session instead of reconnecting.
fn counting_two_cycle_pbap_connector(
    calls: Arc<AtomicUsize>,
) -> PbapConnector<tokio::io::DuplexStream> {
    Box::new(move || {
        calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.next().await;
                t.send(Bytes::from_static(PBAP_CONNECT_RSP)).await.ok();
                serve_one_empty_sync_cycle(&mut t).await;
                serve_one_empty_sync_cycle(&mut t).await;
            });
            pbap_core::client::PbapClient::connect(client_io).await.map_err(SessionError::from)
        })
    })
}

/// Builds a counting PBAP connector whose every call is a fresh connection serving exactly
/// one `sync_contacts` cycle, then ending the fake server task — a second op reused against
/// an already-exhausted connection hits a transport error, modelling a dropped PBAP link.
fn counting_one_cycle_pbap_connector(
    calls: Arc<AtomicUsize>,
) -> PbapConnector<tokio::io::DuplexStream> {
    Box::new(move || {
        calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.next().await;
                t.send(Bytes::from_static(PBAP_CONNECT_RSP)).await.ok();
                serve_one_empty_sync_cycle(&mut t).await;
            });
            pbap_core::client::PbapClient::connect(client_io).await.map_err(SessionError::from)
        })
    })
}

/// Sends a `SyncContacts` op through the actor's real `mpsc` channel and awaits its reply —
/// exercises the actual dispatch path, not a hand-called private method.
async fn sync_contacts(h: &ActorHandles) -> anyhow::Result<ipc::BrokerResponse> {
    let (reply, rx) = tokio::sync::oneshot::channel();
    h.handle.send(DeviceOp::SyncContacts { reply }).await?;
    Ok(rx.await?)
}

/// A second (and third) `SyncContacts` op must reuse the actor's held PBAP session rather
/// than reconnecting — the persistent-session shape `MAP` ops already get, extended to PBAP.
#[tokio::test]
async fn sync_contacts_reuses_persistent_pbap_session() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let calls = Arc::new(AtomicUsize::new(0));
    let h = spawn(
        fake_connector(),
        counting_two_cycle_pbap_connector(calls.clone()),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;

    for _ in 0..2 {
        let resp = sync_contacts(&h).await?;
        assert!(matches!(resp, ipc::BrokerResponse::ContactsSynced { count: 0 }));
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "second SyncContacts reconnected instead of reusing the held PBAP session"
    );
    Ok(())
}

/// Builds a counting PBAP connector whose single connection answers two `ListContacts`
/// request cycles (one `ListvCardObjects` GET each) before the fake server task ends.
fn counting_two_list_pbap_connector(
    calls: Arc<AtomicUsize>,
) -> PbapConnector<tokio::io::DuplexStream> {
    Box::new(move || {
        calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            let (client_io, server_io) = duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.next().await;
                t.send(Bytes::from_static(PBAP_CONNECT_RSP)).await.ok();
                for _ in 0..2 {
                    t.next().await;
                    if let Ok(pkt) = ok_body_packet(b"<vCard-listing></vCard-listing>") {
                        t.send(pkt).await.ok();
                    }
                }
            });
            pbap_core::client::PbapClient::connect(client_io).await.map_err(SessionError::from)
        })
    })
}

/// Sends a `ListContacts` op through the actor's real `mpsc` channel and awaits its reply.
async fn list_contacts(h: &ActorHandles) -> anyhow::Result<ipc::BrokerResponse> {
    let (reply, rx) = tokio::sync::oneshot::channel();
    h.handle.send(DeviceOp::ListContacts { path: None, limit: None, offset: 0, reply }).await?;
    Ok(rx.await?)
}

/// A `ListContacts` op reuses the actor's held PBAP session exactly like `SyncContacts` does —
/// proving the new live-browse ops inherit `run_pbap_op`'s reuse guarantee for free, as intended
/// when it was factored out of `SyncContacts`'s original standalone handling.
#[tokio::test]
async fn list_contacts_reuses_persistent_pbap_session() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let calls = Arc::new(AtomicUsize::new(0));
    let h = spawn(
        fake_connector(),
        counting_two_list_pbap_connector(calls.clone()),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;

    for _ in 0..2 {
        let resp = list_contacts(&h).await?;
        assert!(matches!(resp, ipc::BrokerResponse::ContactEntries(entries) if entries.is_empty()));
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "second ListContacts reconnected instead of reusing the held PBAP session"
    );
    Ok(())
}

/// A PBAP transport error drops only the cached PBAP session (forcing a reconnect on the
/// *next* PBAP op) — it must never mark the unrelated MAP session lost.
#[tokio::test]
async fn pbap_failure_reconnects_independently_of_map_session() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let calls = Arc::new(AtomicUsize::new(0));
    let h = spawn(
        fake_connector(),
        counting_one_cycle_pbap_connector(calls.clone()),
        store,
        Some(Duration::from_secs(60)),
        test_policy(),
    );
    let mut state = h.state.clone();
    state.wait_for(|s| matches!(s, ConnState::Active)).await?;

    let first = sync_contacts(&h).await?;
    assert!(matches!(first, ipc::BrokerResponse::ContactsSynced { count: 0 }));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Second call reuses the same (now exhausted) connection and fails.
    let second = sync_contacts(&h).await?;
    assert!(matches!(second, ipc::BrokerResponse::Failed(_)));
    assert!(matches!(*state.borrow(), ConnState::Active), "MAP session must stay Active");

    // Third call must reconnect PBAP rather than reuse the dead session.
    let third = sync_contacts(&h).await?;
    assert!(matches!(third, ipc::BrokerResponse::ContactsSynced { count: 0 }));
    assert_eq!(calls.load(Ordering::SeqCst), 2, "failed PBAP op must reconnect on the next call");
    Ok(())
}
