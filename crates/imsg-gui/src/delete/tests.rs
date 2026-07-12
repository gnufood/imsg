//! Real-socket tests for `delete`, same fake-broker approach as `crate::daemon`'s own tests.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::MAX_FRAME_LEN;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::*;

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

async fn serve_one(listener: Listener, resp: ipc::BrokerResponse) -> anyhow::Result<()> {
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
async fn delete_returns_broker_text_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:05:01";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Text("deleted H1".to_owned())));

    let got = delete(addr, "H1".to_owned(), "telecom/msg/inbox".to_owned()).await?;

    assert_eq!(got, "deleted H1");
    server.await??;
    Ok(())
}

#[tokio::test]
async fn delete_maps_unreachable_broker_to_write_error() -> anyhow::Result<()> {
    let Err(err) =
        delete("TE:ST:00:00:05:02", "H1".to_owned(), "telecom/msg/inbox".to_owned()).await
    else {
        return Err(anyhow::anyhow!("expected an error when nothing is listening"));
    };
    assert!(matches!(err, broker_client::WriteError::Connect(_)));
    Ok(())
}
