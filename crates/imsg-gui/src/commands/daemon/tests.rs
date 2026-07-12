//! Real-socket tests for `daemon_status`/`broker_status`/`daemon_stop` (same fake-broker
//! approach as `crate::daemon`'s own tests). `daemon_install`/`daemon_uninstall` are
//! intentionally untested here for the same reason `crate::daemon::{install,uninstall}` are —
//! see that module's test doc.

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
async fn daemon_status_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:03:01";
    let resp = ipc::BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: addr.into(),
        persistent: true,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    assert_eq!(daemon_status(addr.to_owned()).await, Some(SessionState::Active));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn broker_status_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:03:02";
    let resp = ipc::BrokerResponse::StatusInfo {
        state: SessionState::Connecting,
        device: addr.into(),
        persistent: false,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    assert_eq!(broker_status(addr.to_owned()).await, Some(SessionState::Connecting));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn daemon_stop_reports_not_running_when_unreachable() -> anyhow::Result<()> {
    let outcome = daemon_stop("TE:ST:00:00:03:03".to_owned()).await?;
    assert_eq!(outcome, crate::daemon::StopOutcome::NotRunning);
    Ok(())
}

#[tokio::test]
async fn daemon_stop_maps_error_response_to_command_error() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:03:04";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Error("no coordinator".to_owned())));

    let result = daemon_stop(addr.to_owned()).await;
    assert!(result.is_err());
    server.await??;
    Ok(())
}
