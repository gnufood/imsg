//! Real-socket tests for [`super::query_persistent`] / [`super::query_state`].

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerRequest, BrokerResponse, SessionState, MAX_FRAME_LEN};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::{query_persistent, query_state};

// binds at the same abstract name connect_raw derives from addr via config::broker_abstract_name
// — the naming scheme is production code, not duplicated here
fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

// listener must be bound before spawning this — binding it here instead would race the
// caller's immediate connect attempt
async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

#[tokio::test]
async fn query_persistent_reports_daemon_mode() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:01:01";
    let resp = BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: addr.into(),
        persistent: true,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    let got = query_persistent(addr).await;

    assert_eq!(got, Some(true));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn query_persistent_is_none_when_unreachable() {
    assert_eq!(query_persistent("TE:ST:00:00:01:02").await, None);
}

#[tokio::test]
async fn query_state_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:01:03";
    let resp = BrokerResponse::StatusInfo {
        state: SessionState::Reconnecting,
        device: addr.into(),
        persistent: false,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    let got = query_state(addr).await;

    assert_eq!(got, Some(SessionState::Reconnecting));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn query_state_is_none_when_unreachable() {
    assert_eq!(query_state("TE:ST:00:00:01:04").await, None);
}
