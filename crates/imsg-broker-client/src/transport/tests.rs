//! Real-socket tests for [`super::send_request`] — no fakes/mocks, a live `interprocess`
//! listener stands in for the broker.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerRequest, BrokerResponse, SessionState, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::send_request;

/// Binds at the same abstract name `connect_raw` derives from `addr` via
/// `config::broker_abstract_name` — the naming scheme is production code, not duplicated here.
fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

/// Accepts one connection, decodes one `BrokerRequest` frame, replies with `resp`.
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

/// `send_request` round-trips a real request/response pair over a real abstract socket.
#[tokio::test]
async fn send_request_roundtrips_through_real_socket() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:00:01";
    let listener = bind_for(addr)?;
    let want_resp = BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: addr.into(),
        persistent: true,
    };
    let server = tokio::spawn(serve_one(listener, want_resp.clone()));

    let resp = send_request(addr, BrokerRequest::Status).await?;

    assert!(matches!(resp, BrokerResponse::StatusInfo { persistent: true, .. }));
    let seen_req = server.await??;
    assert!(matches!(seen_req, BrokerRequest::Status));
    Ok(())
}

/// `send_request` fails when nothing is listening at the abstract name.
#[tokio::test]
async fn send_request_fails_when_unreachable() {
    let result = send_request("TE:ST:00:00:00:02", BrokerRequest::Status).await;
    assert!(result.is_err());
}
