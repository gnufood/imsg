//! Per-connection request handling: readiness gating, one-shot dispatch, watch streaming
//! (delegated to [`watch_conn`]), and `Status` served straight from the connection-state watch.
//!
//! Each accepted connection runs one of these to completion in its own task.

mod watch_conn;

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::StreamExt as _;
use ipc::{BrokerRequest, BrokerResponse, Reason, MAX_FRAME_LEN};
use std::time::Duration;
use tokio::sync::{oneshot, watch};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

use crate::runtime::types::{ConnState, DeviceHandle, DeviceOp};
use watch_conn::handle_watch;

/// Reads one request frame and routes it: `Status` from the state watch, `Watch` to the stream
/// handler, `Shutdown` to the coordinator token (if any), everything else through the readiness
/// gate to the actor.
///
/// `shutdown` is `Some` only for a persistent (daemon-mode) broker — the ephemeral one-shot
/// broker has no coordinator to cancel, so it answers `Shutdown` with an explicit error instead
/// of silently ignoring it.
pub(in crate::runtime) async fn handle_connection<S>(
    stream: S,
    handle: &DeviceHandle,
    state: watch::Receiver<ConnState>,
    device: String,
    readiness_wait: Duration,
    shutdown: Option<&CancellationToken>,
) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);

    let frame = match framed.next().await {
        None => return Ok(()),
        Some(Err(e)) => return Err(e.into()),
        Some(Ok(f)) => f,
    };
    let req: BrokerRequest = match serde_json::from_slice(&frame) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("malformed request: {e}");
            let _ = send_frame(&mut framed, &BrokerResponse::Error(e.to_string())).await;
            return Ok(());
        }
    };

    match req {
        BrokerRequest::Status => {
            let info = BrokerResponse::StatusInfo {
                state: state.borrow().to_wire(),
                device,
                persistent: shutdown.is_some(),
            };
            send_frame(&mut framed, &info).await
        }
        BrokerRequest::Watch => handle_watch(framed, handle, shutdown).await,
        BrokerRequest::Shutdown => handle_shutdown(framed, shutdown).await,
        other => handle_one_shot(framed, handle, state, readiness_wait, other).await,
    }
}

/// Cancels the shutdown coordinator's token, or reports that this broker instance has none.
async fn handle_shutdown<S: tokio::io::AsyncWrite + Unpin>(
    mut framed: Framed<S, LengthDelimitedCodec>,
    shutdown: Option<&CancellationToken>,
) -> Result<()> {
    let Some(token) = shutdown else {
        return send_frame(
            &mut framed,
            &BrokerResponse::Error("shutdown not supported by an ephemeral broker".into()),
        )
        .await;
    };
    token.cancel();
    send_frame(&mut framed, &BrokerResponse::Ok).await
}

/// Resolution of the readiness gate for an operational request.
enum Ready {
    /// Session is `Active` — proceed.
    Active,
    /// Session went terminal — fail with this reason.
    Failed(Reason),
    /// Deadline elapsed before the session became `Active`.
    Timeout,
}

/// Waits up to `deadline` for the session to become `Active`, returning early on a terminal state.
async fn await_ready(state: &mut watch::Receiver<ConnState>, deadline: Duration) -> Ready {
    let wait = state.wait_for(|s| matches!(s, ConnState::Active | ConnState::Failed(_)));
    match tokio::time::timeout(deadline, wait).await {
        Ok(Ok(s)) => match &*s {
            ConnState::Failed(reason) => Ready::Failed(reason.clone()),
            _ => Ready::Active,
        },
        Ok(Err(_)) => Ready::Failed(Reason::Internal("broker stopped".into())),
        Err(_) => Ready::Timeout,
    }
}

/// Gates on readiness, then converts the request to a [`DeviceOp`], dispatches it, and replies.
async fn handle_one_shot<S: tokio::io::AsyncWrite + Unpin>(
    mut framed: Framed<S, LengthDelimitedCodec>,
    handle: &DeviceHandle,
    mut state: watch::Receiver<ConnState>,
    readiness_wait: Duration,
    req: BrokerRequest,
) -> Result<()> {
    match await_ready(&mut state, readiness_wait).await {
        Ready::Active => {}
        Ready::Failed(reason) => {
            return send_frame(&mut framed, &BrokerResponse::Failed(reason)).await
        }
        Ready::Timeout => {
            return send_frame(&mut framed, &BrokerResponse::Failed(Reason::NotReady)).await;
        }
    }
    let (tx, rx) = oneshot::channel();
    let op = match req_to_op(req, tx) {
        Ok(op) => op,
        Err(resp) => return send_frame(&mut framed, &resp).await,
    };
    if handle.send(op).await.is_err() {
        let _ =
            send_frame(&mut framed, &BrokerResponse::Error("broker shutting down".into())).await;
        return Ok(());
    }
    let resp = rx.await.unwrap_or_else(|_| BrokerResponse::Error("actor dropped reply".into()));
    send_frame(&mut framed, &resp).await
}

/// Converts a one-shot [`BrokerRequest`] into a [`DeviceOp`] with its reply channel.
///
/// Returns `Err` for `Status`/`Watch`, which are handled before this point.
fn req_to_op(
    req: BrokerRequest,
    reply: oneshot::Sender<BrokerResponse>,
) -> std::result::Result<DeviceOp, Box<BrokerResponse>> {
    let op = match req {
        BrokerRequest::Sync { folder } => DeviceOp::Sync { folder, reply },
        BrokerRequest::Send { number, message } => DeviceOp::Send { number, message, reply },
        BrokerRequest::Delete { handle, folder } => {
            DeviceOp::Delete { msg_handle: handle, folder, reply }
        }
        BrokerRequest::Backfill => DeviceOp::Backfill { reply },
        BrokerRequest::ListMessages { folder, unread, from, since, limit, offset } => {
            DeviceOp::LiveList { folder, unread, from, since, limit, offset, reply }
        }
        BrokerRequest::GetMessage { handle } => DeviceOp::LiveGet { handle, reply },
        BrokerRequest::Threads => DeviceOp::LiveThreads { reply },
        BrokerRequest::Folders => DeviceOp::LiveFolders { reply },
        BrokerRequest::MarkReadDevice { handle } => DeviceOp::LiveMarkRead { handle, reply },
        BrokerRequest::SendLive { number, message } => {
            DeviceOp::LiveSend { number, message, reply }
        }
        BrokerRequest::SyncContacts => DeviceOp::SyncContacts { reply },
        BrokerRequest::ListContacts { path, limit, offset } => {
            DeviceOp::ListContacts { path, limit, offset, reply }
        }
        BrokerRequest::GetContact { path, handle } => DeviceOp::GetContact { path, handle, reply },
        BrokerRequest::LookupContact { path, number } => {
            DeviceOp::LookupContact { path, number, reply }
        }
        BrokerRequest::PullAllContacts { path, limit, offset } => {
            DeviceOp::PullAllContacts { path, limit, offset, reply }
        }
        BrokerRequest::Status | BrokerRequest::Watch | BrokerRequest::Shutdown => {
            return Err(Box::new(BrokerResponse::Error(
                "internal: routed to one-shot path".into(),
            )));
        }
    };
    Ok(op)
}

/// Serialises `resp` to JSON and writes one length-delimited frame.
///
/// # Errors
///
/// Returns an error if serialisation or the underlying socket write fails.
async fn send_frame<S: tokio::io::AsyncWrite + Unpin>(
    framed: &mut Framed<S, LengthDelimitedCodec>,
    resp: &BrokerResponse,
) -> Result<()> {
    use futures::SinkExt as _;
    let bytes = Bytes::from(serde_json::to_vec(resp).context("serialising response")?);
    framed.send(bytes).await.context("sending response frame")
}

#[cfg(test)]
mod tests;
