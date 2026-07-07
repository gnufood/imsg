//! Unit tests for [`super::handle_connection`]'s dispatch and readiness gating.

use futures::SinkExt as _;
use ipc::SessionState;
use tokio::io::duplex;
use tokio_util::sync::CancellationToken;

use super::*;

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
        handle_connection(server, &handle, state_rx, "AA:BB:CC:DD:EE:FF".into(), secs(1), None)
            .await
    });
    let mut framed = Framed::new(client, codec());
    send_request(&mut framed, &BrokerRequest::Status).await?;
    let frame = framed.next().await.context("broker closed stream")??;
    let resp: BrokerResponse = serde_json::from_slice(&frame)?;
    assert!(matches!(resp, BrokerResponse::StatusInfo { state: SessionState::Reconnecting, .. }));
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
        handle_connection(server, &handle, state_rx, "dev".into(), Duration::from_millis(40), None)
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

/// The ephemeral broker has no coordinator to cancel — `Shutdown` must fail explicitly rather
/// than silently no-op.
#[tokio::test]
async fn shutdown_without_coordinator_errors() -> anyhow::Result<()> {
    let (op_tx, _op_rx) = tokio::sync::mpsc::channel(1);
    let handle = DeviceHandle::from_sender(op_tx);
    let (_state_tx, state_rx) = watch::channel(ConnState::Active);
    let (client, server) = duplex(4096);

    let task = tokio::spawn(async move {
        handle_connection(server, &handle, state_rx, "dev".into(), secs(1), None).await
    });
    let mut framed = Framed::new(client, codec());
    send_request(&mut framed, &BrokerRequest::Shutdown).await?;
    let frame = framed.next().await.context("broker closed stream")??;
    let resp: BrokerResponse = serde_json::from_slice(&frame)?;
    assert!(matches!(resp, BrokerResponse::Error(_)), "unexpected response: {resp:?}");
    let _ = task.await;
    Ok(())
}

/// A daemon-mode `Shutdown` request cancels the coordinator's token and acks `Ok`.
#[tokio::test]
async fn shutdown_with_coordinator_cancels_token() -> anyhow::Result<()> {
    let (op_tx, _op_rx) = tokio::sync::mpsc::channel(1);
    let handle = DeviceHandle::from_sender(op_tx);
    let (_state_tx, state_rx) = watch::channel(ConnState::Active);
    let (client, server) = duplex(4096);
    let token = CancellationToken::new();
    let watched = token.clone();

    let task = tokio::spawn(async move {
        handle_connection(server, &handle, state_rx, "dev".into(), secs(1), Some(&token)).await
    });
    let mut framed = Framed::new(client, codec());
    send_request(&mut framed, &BrokerRequest::Shutdown).await?;
    let frame = framed.next().await.context("broker closed stream")??;
    let resp: BrokerResponse = serde_json::from_slice(&frame)?;
    assert!(matches!(resp, BrokerResponse::Ok), "unexpected response: {resp:?}");
    assert!(watched.is_cancelled());
    let _ = task.await;
    Ok(())
}

/// A live `Watch` connection must exit promptly when the shutdown token cancels, not only when
/// the actor's broadcast sender is dropped — regression guard for a Watch subscriber that used
/// to outlive `accept_and_drain`'s bounded drain timeouts.
#[tokio::test]
async fn watch_exits_promptly_on_shutdown_cancel() -> anyhow::Result<()> {
    let (op_tx, mut op_rx) = tokio::sync::mpsc::channel(4);
    let handle = DeviceHandle::from_sender(op_tx);
    let (_state_tx, state_rx) = watch::channel(ConnState::Active);
    let (client, server) = duplex(4096);
    let (bc_tx, _bc_rx) = broadcast::channel(4);
    let token = CancellationToken::new();
    let watched = token.clone();

    // Fake actor: answers Backfill/Subscribe then holds the subscription open, never sending an
    // event and never closing — only the shutdown token can end the connection's loop.
    tokio::spawn(async move {
        while let Some(op) = op_rx.recv().await {
            match op {
                DeviceOp::Backfill { reply } => {
                    let _ = reply.send(BrokerResponse::Ok);
                }
                DeviceOp::Subscribe { reply } => {
                    let _ = reply.send(bc_tx.subscribe());
                }
                _ => {}
            }
        }
    });

    let task = tokio::spawn(async move {
        handle_connection(server, &handle, state_rx, "dev".into(), secs(1), Some(&token)).await
    });
    let mut framed = Framed::new(client, codec());
    send_request(&mut framed, &BrokerRequest::Watch).await?;
    tokio::time::sleep(Duration::from_millis(20)).await; // let Subscribe land
    watched.cancel();

    tokio::time::timeout(Duration::from_millis(500), task)
        .await
        .context("watch connection did not exit promptly after shutdown cancel")???;
    Ok(())
}

fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}
