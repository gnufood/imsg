use std::sync::Arc;

use bluer::Address;
use bytes::Bytes;
use iroh::endpoint::{presets, Incoming};
use obex_core::TransportError;
use tokio::sync::{broadcast, watch, Semaphore};
use tokio::task::JoinSet;

use super::{iroh_err, Connection, Endpoint, SecretKey, MAP_ALPN, MNS_ALPN, PBAP_ALPN};

/// RFCOMM target the hub proxies MAP/PBAP streams to — bundled since `handle_incoming` and
/// `run_hub` always pass the three together.
#[derive(Clone, Copy)]
struct RfcommTarget {
    addr: Address,
    map_ch: u8,
    pbap_ch: u8,
}

/// Resolves once `cancel`'s owning loop should stop: `cancel` was explicitly set `true`, or its
/// sender was dropped without ever sending. Both are terminal — no further signal can arrive
/// either way — so treating a closed channel as "stop" avoids a busy-poll loop on `changed()`,
/// which resolves immediately (not pending) once the sender is gone.
async fn cancelled(cancel: &mut watch::Receiver<bool>) -> bool {
    loop {
        match cancel.changed().await {
            Ok(()) if *cancel.borrow() => return true,
            Ok(()) => {}
            Err(_) => return true,
        }
    }
}

/// Runs the hub: accepts spoke QUIC connections and routes each by ALPN.
///
/// MAP/PBAP streams are proxied to RFCOMM at `bt_addr` on `map_ch`/`pbap_ch`; at most one active
/// proxy per channel at any time — concurrent connections queue until the previous OBEX session
/// completes. MNS spokes receive every payload published to `mns_events` (sourced externally —
/// the RFCOMM MNS listener lives in the session layer, not here). Returns once `cancel` holds
/// `true`. Per-spoke failures are logged and never terminate the hub. Does not register a
/// Bluetooth SDP profile — the hub is a QUIC endpoint, not a `BlueZ` service.
///
/// # Errors
///
/// Returns [`TransportError::External`] if the hub endpoint cannot bind.
pub async fn run_hub(
    key: SecretKey,
    bt_addr: Address,
    map_ch: u8,
    pbap_ch: u8,
    mns_events: broadcast::Sender<Bytes>,
    mut cancel: watch::Receiver<bool>,
) -> Result<(), TransportError> {
    let target = RfcommTarget { addr: bt_addr, map_ch, pbap_ch };
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(key)
        .alpns(vec![MAP_ALPN.to_vec(), PBAP_ALPN.to_vec(), MNS_ALPN.to_vec()])
        .bind()
        .await
        .map_err(iroh_err)?;
    let map_sem = Arc::new(Semaphore::new(1));
    let pbap_sem = Arc::new(Semaphore::new(1));
    let mut tasks = JoinSet::new();
    loop {
        tokio::select! {
            _ = cancelled(&mut cancel) => break,
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else { break };
                let events = mns_events.subscribe();
                let map_sem = Arc::clone(&map_sem);
                let pbap_sem = Arc::clone(&pbap_sem);
                let cancel = cancel.clone();
                tasks.spawn(async move {
                    if let Err(e) =
                        handle_incoming(incoming, target, events, map_sem, pbap_sem, cancel).await
                    {
                        tracing::warn!("hub spoke connection failed: {e}");
                    }
                });
            }
        }
    }
    // Drain in-flight proxy tasks so copy_bidirectional completes naturally
    // before Endpoint::close tears down the QUIC connections.
    while tasks.join_next().await.is_some() {}
    endpoint.close().await;
    Ok(())
}

async fn handle_incoming(
    incoming: Incoming,
    target: RfcommTarget,
    mns_events: broadcast::Receiver<Bytes>,
    map_sem: Arc<Semaphore>,
    pbap_sem: Arc<Semaphore>,
    cancel: watch::Receiver<bool>,
) -> Result<(), TransportError> {
    let mut accepting = incoming.accept().map_err(iroh_err)?;
    let alpn = accepting.alpn().await.map_err(iroh_err)?;
    let conn = accepting.await.map_err(iroh_err)?;
    match alpn.as_slice() {
        MAP_ALPN => proxy_rfcomm(conn, target.addr, target.map_ch, map_sem).await,
        PBAP_ALPN => proxy_rfcomm(conn, target.addr, target.pbap_ch, pbap_sem).await,
        MNS_ALPN => stream_mns(conn, mns_events, cancel).await,
        // iroh only surfaces connections whose ALPN was advertised at bind.
        _ => Ok(()),
    }
}

async fn proxy_rfcomm(
    conn: Connection,
    bt_addr: Address,
    channel: u8,
    sem: Arc<Semaphore>,
) -> Result<(), TransportError> {
    let _permit = sem
        .acquire()
        .await
        .map_err(|_| TransportError::External("rfcomm semaphore closed".into()))?;
    let (send, recv) = conn.accept_bi().await.map_err(iroh_err)?;
    let mut quic = tokio::io::join(recv, send);
    let mut rfcomm =
        crate::rfcomm::connect(bt_addr, channel, crate::rfcomm::DEFAULT_BT_CONNECTED_GATE, None)
            .await?;
    let result = tokio::io::copy_bidirectional(&mut quic, &mut rfcomm).await;
    match &result {
        Ok((up, down)) => tracing::info!("hub proxy ch{channel}: {up}↑ {down}↓ bytes proxied"),
        Err(e) => tracing::info!("hub proxy ch{channel}: closed ({e})"),
    }
    Ok(())
}

async fn stream_mns(
    conn: Connection,
    mut events: broadcast::Receiver<Bytes>,
    mut cancel: watch::Receiver<bool>,
) -> Result<(), TransportError> {
    // Hub accepts the spoke-initiated stream. _recv dropped — spoke sends nothing.
    let (mut send, _recv) = conn.accept_bi().await.map_err(iroh_err)?;
    loop {
        tokio::select! {
            // Without this branch the loop only exits on broadcast-channel closure or
            // connection loss, so run_hub's shutdown drain would hang while any MNS
            // spoke stays connected.
            _ = cancelled(&mut cancel) => break,
            event = events.recv() => match event {
                Ok(payload) => {
                    let len = u32::try_from(payload.len()).map_err(|_| {
                        TransportError::External("MNS payload exceeds 4 GiB".into())
                    })?;
                    send.write_all(&len.to_be_bytes()).await.map_err(iroh_err)?;
                    send.write_all(&payload).await.map_err(iroh_err)?;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("MNS broadcast lagged; {n} events dropped");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::watch;

    use super::cancelled;

    // bounded timeout: a regression back to polling `cancel.changed()` directly and discarding
    // its Result would busy-spin forever here instead of returning promptly (SAFE-01/SAFE-02).
    async fn resolves_within(
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<bool, tokio::time::error::Elapsed> {
        tokio::time::timeout(Duration::from_millis(200), cancelled(cancel)).await
    }

    #[tokio::test]
    async fn resolves_true_once_sender_sends_true() -> Result<(), tokio::time::error::Elapsed> {
        let (tx, mut rx) = watch::channel(false);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = tx.send(true);
        });
        assert!(resolves_within(&mut rx).await?);
        Ok(())
    }

    #[tokio::test]
    async fn ignores_intermediate_false_sends() -> Result<(), tokio::time::error::Elapsed> {
        let (tx, mut rx) = watch::channel(false);
        tokio::spawn(async move {
            let _ = tx.send(false);
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = tx.send(true);
        });
        assert!(resolves_within(&mut rx).await?);
        Ok(())
    }

    #[tokio::test]
    async fn resolves_true_when_sender_dropped_without_sending(
    ) -> Result<(), tokio::time::error::Elapsed> {
        let (tx, mut rx) = watch::channel(false);
        drop(tx);
        assert!(resolves_within(&mut rx).await?);
        Ok(())
    }
}
