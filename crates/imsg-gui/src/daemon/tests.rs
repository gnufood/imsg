//! Real-socket tests for `status`/`stop` (same fake-broker approach as `imsg-broker-client`'s
//! own `query`/`tests.rs` — a real abstract-namespace listener, not a mock). `install`/
//! `uninstall` are intentionally untested against a real OS service manager here, matching
//! `imsg-service`'s own test suite (`crates/imsg-service/src/tests.rs`), which never calls
//! `install`/`uninstall`/`status` for real either — doing so would register an actual
//! systemd/launchd/etc. unit on the machine running the tests.

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

/// Accepts one connection, decodes the request, replies with `resp`.
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

#[test]
fn level_maps_system_flag() {
    assert_eq!(level(true), service::ServiceLevel::System);
    assert_eq!(level(false), service::ServiceLevel::User);
}

#[tokio::test]
async fn status_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:01";
    let resp = ipc::BrokerResponse::StatusInfo {
        state: ipc::SessionState::Active,
        device: addr.into(),
        persistent: true,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    assert_eq!(status(addr).await, Some(ipc::SessionState::Active));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn status_is_none_when_unreachable() {
    assert_eq!(status("TE:ST:00:00:02:02").await, None);
}

#[tokio::test]
async fn stop_reports_not_running_when_unreachable() -> anyhow::Result<()> {
    assert_eq!(stop("TE:ST:00:00:02:03").await?, StopOutcome::NotRunning);
    Ok(())
}

#[tokio::test]
async fn stop_reports_stopping_on_ok_response() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:04";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, ipc::BrokerResponse::Ok));

    assert_eq!(stop(addr).await?, StopOutcome::Stopping);
    server.await??;
    Ok(())
}

#[tokio::test]
async fn stop_maps_error_response_to_rejected() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:05";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Error("no coordinator".to_owned())));

    let Err(err) = stop(addr).await else {
        return Err(anyhow::anyhow!("expected an error for an Error response"));
    };
    assert!(matches!(err, StopError::Rejected(msg) if msg == "no coordinator"));
    server.await??;
    Ok(())
}
