//! Real-socket tests for [`super::folders`], same fake-broker approach as `contacts/tests.rs`.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerRequest, BrokerResponse, FolderDto, Reason, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::{folders, ReadError};
use crate::CallError;

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<BrokerRequest> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let req: BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(req)
}

#[tokio::test]
async fn folders_sends_folders_request_and_returns_rows() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:05:01";
    let listener = bind_for(addr)?;
    let rows = vec![FolderDto { name: "inbox".to_owned() }, FolderDto { name: "sent".to_owned() }];
    let server = tokio::spawn(serve_one(listener, BrokerResponse::Folders(rows.clone())));

    let got = folders(addr).await?;

    assert_eq!(got, rows);
    assert!(matches!(server.await??, BrokerRequest::Folders));
    Ok(())
}

#[tokio::test]
async fn folders_maps_failed_response_to_call_error() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:05:02";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, BrokerResponse::Failed(Reason::DeviceUnreachable)));

    let Err(err) = folders(addr).await else {
        return Err(anyhow::anyhow!("expected an error for a Failed response"));
    };
    assert!(matches!(err, ReadError::Call(CallError::Failed(Reason::DeviceUnreachable))));
    server.await??;
    Ok(())
}

/// Nothing is listening, so this must surface as a connect error the caller can act on rather
/// than an empty listing.
#[tokio::test]
async fn folders_maps_unreachable_broker_to_connect_error() -> anyhow::Result<()> {
    let Err(err) = folders("TE:ST:00:00:05:03").await else {
        return Err(anyhow::anyhow!("expected an error when nothing is listening"));
    };
    assert!(matches!(err, ReadError::Connect(_)));
    Ok(())
}
