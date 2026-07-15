//! Real-socket, real-`Store` tests — same fake-broker approach as `crate::send`'s and
//! `daemon::provision`'s own tests, no mocks.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerResponse, MAX_FRAME_LEN};
use secrecy::SecretBox;
use store::Store;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::*;

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: ipc::BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

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
