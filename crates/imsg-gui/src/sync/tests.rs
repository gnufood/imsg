//! Real-socket, real-`Store` tests — same fake-broker approach as `crate::send`'s and
//! `daemon::provision`'s own tests, no mocks.

use ipc::BrokerResponse;

use crate::test_support::{bind_for, fake_store, serve_one};

use super::*;

#[tokio::test]
async fn ensure_synced_short_circuits_when_already_enabled() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.set_meta("sync_enabled", "true").await?;

    // No listener bound at this address — if `ensure_synced` tried to reach the broker anyway,
    // this would fail. It must not.
    ensure_synced(&store, "TE:ST:00:00:05:01").await?;
    Ok(())
}

#[tokio::test]
async fn ensure_synced_triggers_sync_and_marks_enabled_when_unset() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let addr = "TE:ST:00:00:05:02";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, BrokerResponse::Text("synced".to_owned())));

    ensure_synced(&store, addr).await?;

    assert_eq!(store.get_meta("sync_enabled").await?.as_deref(), Some("true"));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn ensure_synced_leaves_meta_unset_when_broker_unreachable() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;

    // No listener bound — the broker call must fail.
    let result = ensure_synced(&store, "TE:ST:00:00:05:03").await;
    assert!(matches!(result, Err(SyncError::Sync(_))));
    assert_eq!(store.get_meta("sync_enabled").await?, None);
    Ok(())
}

#[tokio::test]
async fn ensure_synced_maps_broker_rejection_to_sync_error() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let addr = "TE:ST:00:00:05:04";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, BrokerResponse::Error("no MAP session".to_owned())));

    let result = ensure_synced(&store, addr).await;
    assert!(matches!(result, Err(SyncError::Sync(_))));
    assert_eq!(store.get_meta("sync_enabled").await?, None);
    server.await??;
    Ok(())
}
