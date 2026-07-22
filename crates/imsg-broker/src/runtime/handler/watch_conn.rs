//! The `Watch` streaming connection path — split out of `super` to keep that module under the
//! size ceiling.

use anyhow::Result;
use ipc::BrokerResponse;
use tokio::sync::{broadcast, oneshot};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::sync::CancellationToken;

use super::send_frame;
use crate::runtime::types::{DeviceHandle, DeviceOp};

/// Handles a `Watch` connection: pre-flight backfill → subscribe → stream events.
///
/// Watch is not readiness-gated — it subscribes immediately (counting as demand that keeps the
/// broker alive) and events flow once the session is `Active`. Lagged subscribers trigger a
/// catch-up backfill rather than disconnecting. Races the event stream against `shutdown` so a
/// live subscriber unsubscribes and exits as soon as the coordinator cancels, instead of only
/// when the actor itself tears down — otherwise a connected `Watch` client would hold the actor's
/// op channel open indefinitely and block the daemon's bounded drain.
pub(super) async fn handle_watch<S: tokio::io::AsyncWrite + Unpin>(
    mut framed: Framed<S, LengthDelimitedCodec>,
    handle: &DeviceHandle,
    shutdown: Option<&CancellationToken>,
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
        let resp = tokio::select! {
            () = cancelled(shutdown) => break,
            recv = event_rx.recv() => match recv {
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
            },
        };
        if send_frame(&mut framed, &resp).await.is_err() {
            break;
        }
    }
    let _ = handle.send(DeviceOp::Unsubscribe).await;
    Ok(())
}

/// Resolves when `token` cancels; pending forever for an ephemeral broker with no coordinator to
/// race against, so `tokio::select!` falls through to the other branch unconditionally.
async fn cancelled(token: Option<&CancellationToken>) {
    match token {
        Some(t) => t.cancelled().await,
        None => std::future::pending().await,
    }
}
