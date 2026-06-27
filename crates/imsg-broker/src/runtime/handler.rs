//! Per-connection request handling: readiness gating, one-shot dispatch, watch streaming, and
//! `Status` served straight from the connection-state watch.
//!
//! Each accepted connection runs one of these to completion in its own task.

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::StreamExt as _;
use ipc::{BrokerRequest, BrokerResponse, Reason, MAX_FRAME_LEN};
use std::time::Duration;
use tokio::sync::{broadcast, oneshot, watch};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::runtime::types::{ConnState, DeviceHandle, DeviceOp};

/// Reads one request frame and routes it: `Status` from the state watch, `Watch` to the stream
/// handler, everything else through the readiness gate to the actor.
pub(in crate::runtime) async fn handle_connection<S>(
    stream: S,
    handle: &DeviceHandle,
    state: watch::Receiver<ConnState>,
    device: String,
    readiness_wait: Duration,
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
            let info = BrokerResponse::StatusInfo { state: state.borrow().to_wire(), device };
            send_frame(&mut framed, &info).await
        }
        BrokerRequest::Watch => handle_watch(framed, handle).await,
        other => handle_one_shot(framed, handle, state, readiness_wait, other).await,
    }
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

/// Handles a `Watch` connection: pre-flight backfill → subscribe → stream events.
///
/// Watch is not readiness-gated — it subscribes immediately (counting as demand that keeps the
/// broker alive) and events flow once the session is `Active`. Lagged subscribers trigger a
/// catch-up backfill rather than disconnecting.
async fn handle_watch<S: tokio::io::AsyncWrite + Unpin>(
    mut framed: Framed<S, LengthDelimitedCodec>,
    handle: &DeviceHandle,
) -> Result<()> {
    let (bf_tx, bf_rx) = oneshot::channel();
    if handle.send(DeviceOp::Backfill { reply: bf_tx }).await.is_ok() {
        let _ = bf_rx.await;
    }
    let (sub_tx, sub_rx) = oneshot::channel();
    if handle.send(DeviceOp::Subscribe { reply: sub_tx }).await.is_err() {
        return send_frame(&mut framed, &BrokerResponse::Error("broker shutting down".into()))
            .await;
    }
    let Ok(mut event_rx) = sub_rx.await else {
        return Ok(());
    };

    loop {
        let resp = match event_rx.recv().await {
            Ok(ev) => BrokerResponse::WatchEvent(ev),
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("watch subscriber lagged {n} events — backfilling");
                let (tx, rx) = oneshot::channel();
                if handle.send(DeviceOp::Backfill { reply: tx }).await.is_ok() {
                    let _ = rx.await;
                }
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => break,
        };
        if send_frame(&mut framed, &resp).await.is_err() {
            break;
        }
    }
    let _ = handle.send(DeviceOp::Unsubscribe).await;
    Ok(())
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
        BrokerRequest::MarkReadDevice { handle } => DeviceOp::LiveMarkRead { handle, reply },
        BrokerRequest::SendLive { number, message } => {
            DeviceOp::LiveSend { number, message, reply }
        }
        BrokerRequest::Status | BrokerRequest::Watch => {
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
mod tests {
    use super::*;
    use futures::SinkExt as _;
    use ipc::SessionState;
    use tokio::io::duplex;

    fn codec() -> LengthDelimitedCodec {
        LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec()
    }

    async fn send_request<S: tokio::io::AsyncWrite + Unpin>(
        framed: &mut Framed<S, LengthDelimitedCodec>,
        req: &BrokerRequest,
    ) -> anyhow::Result<()> {
        let bytes = Bytes::from(serde_json::to_vec(req)?);
        framed.send(bytes).await?;
        Ok(())
    }

    #[tokio::test]
    async fn await_ready_resolves_active_immediately() {
        let (_tx, mut rx) = watch::channel(ConnState::Active);
        assert!(matches!(await_ready(&mut rx, Duration::from_millis(50)).await, Ready::Active));
    }

    #[tokio::test]
    async fn await_ready_propagates_failed_reason() {
        let (_tx, mut rx) = watch::channel(ConnState::Failed(Reason::ConnectionRefused));
        assert!(matches!(
            await_ready(&mut rx, Duration::from_millis(50)).await,
            Ready::Failed(Reason::ConnectionRefused)
        ));
    }

    #[tokio::test]
    async fn await_ready_times_out_while_connecting() {
        let (_tx, mut rx) = watch::channel(ConnState::Connecting);
        assert!(matches!(await_ready(&mut rx, Duration::from_millis(20)).await, Ready::Timeout));
    }

    /// Status is answered from the state watch even while `Reconnecting` and with no op consumer —
    /// proving it bypasses the actor op channel and never waits.
    #[tokio::test]
    async fn status_served_from_state_watch() -> anyhow::Result<()> {
        let (op_tx, _op_rx) = tokio::sync::mpsc::channel(1);
        let handle = DeviceHandle::from_sender(op_tx);
        let (state_tx, state_rx) = watch::channel(ConnState::Reconnecting);
        let (client, server) = duplex(4096);

        let task = tokio::spawn(async move {
            handle_connection(server, &handle, state_rx, "AA:BB:CC:DD:EE:FF".into(), secs(1)).await
        });
        let mut framed = Framed::new(client, codec());
        send_request(&mut framed, &BrokerRequest::Status).await?;
        let frame = framed.next().await.context("broker closed stream")??;
        let resp: BrokerResponse = serde_json::from_slice(&frame)?;
        assert!(matches!(
            resp,
            BrokerResponse::StatusInfo { state: SessionState::Reconnecting, .. }
        ));
        drop(state_tx);
        let _ = task.await;
        Ok(())
    }

    /// An operational request returns `NotReady` when the session never reaches `Active` within the
    /// readiness deadline.
    #[tokio::test]
    async fn one_shot_times_out_as_not_ready() -> anyhow::Result<()> {
        let (op_tx, _op_rx) = tokio::sync::mpsc::channel(1);
        let handle = DeviceHandle::from_sender(op_tx);
        let (_state_tx, state_rx) = watch::channel(ConnState::Connecting);
        let (client, server) = duplex(4096);

        let task = tokio::spawn(async move {
            handle_connection(server, &handle, state_rx, "dev".into(), Duration::from_millis(40))
                .await
        });
        let mut framed = Framed::new(client, codec());
        send_request(&mut framed, &BrokerRequest::Sync { folder: None }).await?;
        let frame = framed.next().await.context("broker closed stream")??;
        let resp: BrokerResponse = serde_json::from_slice(&frame)?;
        assert!(matches!(resp, BrokerResponse::Failed(Reason::NotReady)));
        let _ = task.await;
        Ok(())
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }
}
