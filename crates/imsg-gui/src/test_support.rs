//! Shared real-socket/real-`Store` test fixtures.
//!
//! A real abstract-namespace listener speaking the broker wire protocol (not a mock), and a
//! fixed-key temp-dir `Store`. Every module's own `tests.rs` that needs a fake broker or a
//! throwaway store reuses these instead of redefining them.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::MAX_FRAME_LEN;
use secrecy::SecretBox;
use store::Store;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

/// Binds an abstract-namespace listener for `addr` — the same namespace `broker_client` connects to.
pub(crate) fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

/// Accepts one connection, decodes the request, replies with `resp`.
pub(crate) async fn serve_one(listener: Listener, resp: ipc::BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: ipc::BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

/// Opens a throwaway `Store` in a fresh temp dir with a fixed all-zero test key. The returned
/// `TempDir` must outlive the store — dropping it deletes the database file.
pub(crate) async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}
