//! Real-socket tests for `send`, same fake-broker approach as `crate::send`'s own tests.

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
async fn send_returns_broker_text_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:06:01";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, ipc::BrokerResponse::Text("sent".to_owned())));

    let got = send(addr.to_owned(), "+15550001".to_owned(), "hi".to_owned()).await?;

    assert_eq!(got, "sent");
    server.await??;
    Ok(())
}

#[tokio::test]
async fn send_maps_unreachable_broker_to_command_error() -> anyhow::Result<()> {
    let result =
        send("TE:ST:00:00:06:02".to_owned(), "+15550001".to_owned(), "hi".to_owned()).await;
    assert!(result.is_err());
    Ok(())
}
