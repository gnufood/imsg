//! Unit tests for [`super::accept_and_drain`] — the daemon's graceful stop path.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::{
    tokio::Stream as IpcStream, ConnectOptions, GenericNamespaced, ListenerOptions, ToNsName as _,
};
use ipc::{BrokerRequest, BrokerResponse, MAX_FRAME_LEN};
use secrecy::SecretBox;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::*;

const MAP_CONNECT_RSP: &[u8] =
    include_bytes!("../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const NOTIF_REG_OK: &[u8] = &[0xA0, 0x00, 0x03];

/// Same fake OBEX/MAP connector as `server::tests` — no real Bluetooth involved.
fn fake_connector() -> Connector<tokio::io::DuplexStream> {
    Box::new(|| {
        Box::pin(async {
            let (client_io, server_io) = tokio::io::duplex(4096);
            tokio::spawn(async move {
                let mut t = obex_core::wrap(server_io);
                t.send(Bytes::from_static(MAP_CONNECT_RSP)).await.ok();
                t.next().await;
                t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                while t.next().await.is_some() {
                    t.send(Bytes::from_static(NOTIF_REG_OK)).await.ok();
                }
            });
            session::lifecycle::establish_map_session(client_io).await
        })
    })
}

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn test_policy() -> ConnectPolicy {
    ConnectPolicy {
        initial_backoff: Duration::from_millis(1),
        max_backoff: Duration::from_millis(2),
        max_attempts: 2,
        startup_budget: Duration::from_secs(5),
    }
}

/// Connects to `name` and returns a framed client for sending [`BrokerRequest`]s directly,
/// mirroring `cli/src/commands/broker/client.rs`'s wire handling.
async fn connect(name: &str) -> anyhow::Result<Framed<IpcStream, LengthDelimitedCodec>> {
    let ns = name.to_ns_name::<GenericNamespaced>()?;
    let stream = ConnectOptions::new().name(ns).connect_tokio().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    Ok(Framed::new(stream, codec))
}

async fn roundtrip(
    framed: &mut Framed<IpcStream, LengthDelimitedCodec>,
    req: &BrokerRequest,
) -> anyhow::Result<BrokerResponse> {
    framed.send(Bytes::from(serde_json::to_vec(req)?)).await?;
    let frame =
        framed.next().await.ok_or_else(|| anyhow::anyhow!("broker closed without replying"))??;
    Ok(serde_json::from_slice(&frame)?)
}

/// The whole point of the coordinator: with idle disabled (daemon mode), nothing but an
/// external cancellation can stop `accept_and_drain` — prove cancelling the token is enough,
/// and that it converges within a bounded time rather than hanging.
#[tokio::test]
async fn cancel_stops_accept_and_drain_even_with_no_connections() -> anyhow::Result<()> {
    let name = "imsg/broker/test-shutdown-cancel-empty".to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_tokio()?;
    let (store, _dir) = fake_store().await?;
    let handles = super::super::actor::spawn(fake_connector(), store, None, test_policy());
    let token = CancellationToken::new();
    let cancel = token.clone();

    let task = tokio::spawn(async move {
        accept_and_drain(
            handles,
            &listener,
            "test-device".to_owned(),
            Duration::from_millis(50),
            token,
        )
        .await
    });
    cancel.cancel();

    tokio::time::timeout(Duration::from_millis(500), task)
        .await
        .context("accept_and_drain did not converge after cancellation")???;
    Ok(())
}

/// A `Shutdown` request's whole reason to exist: an in-flight connection must be served
/// normally (not aborted) even though it's the one that triggers the coordinator to stop
/// accepting further connections and drain.
#[tokio::test]
async fn shutdown_request_is_served_then_drain_converges() -> anyhow::Result<()> {
    let name = "imsg/broker/test-shutdown-request".to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_tokio()?;
    let (store, _dir) = fake_store().await?;
    let handles = super::super::actor::spawn(fake_connector(), store, None, test_policy());
    let token = CancellationToken::new();

    let task = tokio::spawn(async move {
        accept_and_drain(
            handles,
            &listener,
            "test-device".to_owned(),
            Duration::from_millis(50),
            token,
        )
        .await
    });

    let mut client = connect("imsg/broker/test-shutdown-request").await?;
    let resp = roundtrip(&mut client, &BrokerRequest::Shutdown).await?;
    assert!(matches!(resp, BrokerResponse::Ok), "unexpected response: {resp:?}");

    tokio::time::timeout(Duration::from_millis(500), task)
        .await
        .context("accept_and_drain did not converge after a Shutdown request")???;
    Ok(())
}
