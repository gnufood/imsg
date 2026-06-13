//! iroh QUIC hub/spoke transport — replaces the TCP bridge for remote machines.
//!
//! The hub owns the RFCOMM link and runs [`run_hub`]; spokes reach it over QUIC via the hub's
//! [`EndpointId`]. Each profile rides its own ALPN-tagged bidirectional stream: MAP and PBAP
//! requests are proxied spoke→hub into RFCOMM, MNS events are fanned hub→spoke.

use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use bluer::Address;
use bytes::Bytes;
use iroh::endpoint::{presets, Incoming, VarInt};
use obex_core::{wrap, ObexTransport, TransportError};
use tokio::io::{AsyncRead, AsyncWrite, Join, ReadBuf};
use tokio::sync::{broadcast, watch, Semaphore};

pub use iroh::endpoint::{Connection, RecvStream, SendStream};
pub use iroh::{Endpoint, EndpointAddr, EndpointId, SecretKey};

/// ALPN for spoke→hub MAP request streams.
pub const MAP_ALPN: &[u8] = b"imsg-map/1";
/// ALPN for spoke→hub PBAP request streams.
pub const PBAP_ALPN: &[u8] = b"imsg-pbap/1";
/// ALPN for hub→spoke MNS event streams.
pub const MNS_ALPN: &[u8] = b"imsg-mns/1";

/// Bidirectional QUIC stream presented to OBEX framing as a single duplex I/O object.
pub type SpokeStream = Join<RecvStream, SendStream>;

/// Hub-side OBEX stream that keeps its [`Connection`] alive for the session duration.
///
/// Wraps [`SpokeStream`] and owns the underlying [`Connection`] as a drop guard. Dropping
/// `Connection` tears down all QUIC streams through it immediately — this wrapper prevents
/// that until the OBEX session completes.
pub struct HubStream {
    inner: SpokeStream,
    conn: Connection,
}

impl HubStream {
    /// Pairs `inner` with `conn` so both are dropped together.
    #[must_use]
    pub const fn new(inner: SpokeStream, conn: Connection) -> Self {
        Self { inner, conn }
    }
}

impl Drop for HubStream {
    fn drop(&mut self) {
        self.conn.close(VarInt::from_u32(0), b"");
    }
}

impl AsyncRead for HubStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for HubStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// MNS subscription stream that keeps its [`Connection`] alive for the session duration.
///
/// Wraps [`RecvStream`] and owns the underlying [`Connection`] as a drop guard. Dropping
/// `Connection` tears down all QUIC streams through it immediately — this wrapper prevents
/// that until the event loop completes. Both `RecvStream` and `Connection` are [`Unpin`], so
/// this type is [`Unpin`] and can be passed directly to [`tokio::io::AsyncReadExt`] methods.
pub struct HubRecvStream {
    inner: RecvStream,
    conn: Connection,
}

impl HubRecvStream {
    /// Pairs `inner` with `conn` so both are dropped together.
    #[must_use]
    pub const fn new(inner: RecvStream, conn: Connection) -> Self {
        Self { inner, conn }
    }
}

impl Drop for HubRecvStream {
    fn drop(&mut self) {
        self.conn.close(VarInt::from_u32(0), b"");
    }
}

impl AsyncRead for HubRecvStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

fn iroh_err<E: std::fmt::Display>(e: E) -> TransportError {
    TransportError::External(e.to_string())
}

/// Loads a persisted hub secret key from `path`, or generates, persists, and returns a new one.
///
/// The file holds exactly 32 raw ed25519 seed bytes. Parent directories are created when absent
/// and a freshly generated key is written with mode `0600` before returning. Does not verify
/// that `path` is on an encrypted volume — secure placement is the caller's responsibility.
///
/// # Errors
///
/// Returns [`TransportError::Io`] on directory creation, read, or write failure, and
/// [`TransportError::External`] if an existing key file is not exactly 32 bytes.
pub async fn load_or_create_key(path: &Path) -> Result<SecretKey, TransportError> {
    match tokio::fs::read(path).await {
        Ok(bytes) => {
            let seed = <[u8; 32]>::try_from(bytes.as_slice())
                .map_err(|_| TransportError::External("hub key file is not 32 bytes".into()))?;
            return Ok(SecretKey::from_bytes(&seed));
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let key = SecretKey::generate();
    tokio::fs::write(path, key.to_bytes()).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    Ok(key)
}

/// Binds an ephemeral spoke endpoint with the n0 preset (relay + discovery enabled).
/// Reuse one endpoint for all of a spoke's MAP/PBAP/MNS connections.
///
/// # Errors
///
/// Returns [`TransportError::External`] if no UDP socket can be bound.
pub async fn bind_spoke() -> Result<Endpoint, TransportError> {
    Endpoint::bind(presets::N0).await.map_err(iroh_err)
}

async fn spokeconnect(
    endpoint: &Endpoint,
    hub: EndpointAddr,
    alpn: &[u8],
) -> Result<ObexTransport<SpokeStream>, TransportError> {
    let conn = endpoint.connect(hub, alpn).await.map_err(iroh_err)?;
    let (send, recv) = conn.open_bi().await.map_err(iroh_err)?;
    Ok(wrap(tokio::io::join(recv, send)))
}

/// Opens an OBEX-framed MAP request stream to the hub over ALPN [`MAP_ALPN`].
///
/// `endpoint` is caller-owned (see [`bind_spoke`]). `hub` accepts an [`EndpointId`] — resolved
/// via discovery/relay — or a full [`EndpointAddr`] with direct addresses. Performs no OBEX
/// handshake; the caller drives CONNECT on the returned transport.
///
/// # Errors
///
/// Returns [`TransportError::External`] if the connection or stream cannot be established.
pub async fn connect_map_hub(
    endpoint: &Endpoint,
    hub: impl Into<EndpointAddr>,
) -> Result<ObexTransport<SpokeStream>, TransportError> {
    spokeconnect(endpoint, hub.into(), MAP_ALPN).await
}

/// Identical contract to [`connect_map_hub`] but for the PBAP profile.
///
/// # Errors
///
/// Returns [`TransportError::External`] if the connection or stream cannot be established.
pub async fn connect_pbap_hub(
    endpoint: &Endpoint,
    hub: impl Into<EndpointAddr>,
) -> Result<ObexTransport<SpokeStream>, TransportError> {
    spokeconnect(endpoint, hub.into(), PBAP_ALPN).await
}

/// Opens a persistent MNS subscription stream to the hub over ALPN [`MNS_ALPN`].
///
/// Spoke initiates the bidirectional stream via [`Connection::open_bi`], signalling explicit
/// intent to subscribe. The hub calls [`Connection::accept_bi`] and writes MAP event-report
/// payloads to its send half. The spoke's send half is dropped immediately — MNS is hub→spoke
/// only; the drop sends `STOP_SENDING` so the hub knows no spoke→hub data will arrive.
///
/// Each event is framed as a 4-byte big-endian length prefix followed by the raw event-report
/// bytes. Callers must read the 4-byte header and then exactly that many payload bytes per event.
///
/// `endpoint` is caller-owned (see [`bind_spoke`]). Returns a [`HubRecvStream`] that bundles
/// the receive half and the [`Connection`] drop guard — the connection stays alive until the
/// stream is dropped.
///
/// # Errors
///
/// Returns [`TransportError::External`] if the connection cannot be established or the stream
/// cannot be opened.
pub async fn connect_mns_hub(
    endpoint: &Endpoint,
    hub: impl Into<EndpointAddr>,
) -> Result<HubRecvStream, TransportError> {
    let conn = endpoint.connect(hub.into(), MNS_ALPN).await.map_err(iroh_err)?;
    // Spoke initiates; _send dropped to signal no spoke→hub data expected.
    let (_send, recv) = conn.open_bi().await.map_err(iroh_err)?;
    Ok(HubRecvStream::new(recv, conn))
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
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(key)
        .alpns(vec![MAP_ALPN.to_vec(), PBAP_ALPN.to_vec(), MNS_ALPN.to_vec()])
        .bind()
        .await
        .map_err(iroh_err)?;
    let map_sem = Arc::new(Semaphore::new(1));
    let pbap_sem = Arc::new(Semaphore::new(1));
    loop {
        tokio::select! {
            _ = cancel.changed() => if *cancel.borrow() { break },
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else { break };
                let events = mns_events.subscribe();
                let map_sem = Arc::clone(&map_sem);
                let pbap_sem = Arc::clone(&pbap_sem);
                tokio::spawn(async move {
                    if let Err(e) = handle_incoming(
                        incoming, bt_addr, map_ch, pbap_ch, events, map_sem, pbap_sem,
                    ).await {
                        tracing::warn!("hub spoke connection failed: {e}");
                    }
                });
            }
        }
    }
    endpoint.close().await;
    Ok(())
}

async fn handle_incoming(
    incoming: Incoming,
    bt_addr: Address,
    map_ch: u8,
    pbap_ch: u8,
    mns_events: broadcast::Receiver<Bytes>,
    map_sem: Arc<Semaphore>,
    pbap_sem: Arc<Semaphore>,
) -> Result<(), TransportError> {
    let mut accepting = incoming.accept().map_err(iroh_err)?;
    let alpn = accepting.alpn().await.map_err(iroh_err)?;
    let conn = accepting.await.map_err(iroh_err)?;
    match alpn.as_slice() {
        MAP_ALPN => proxy_rfcomm(conn, bt_addr, map_ch, map_sem).await,
        PBAP_ALPN => proxy_rfcomm(conn, bt_addr, pbap_ch, pbap_sem).await,
        MNS_ALPN => stream_mns(conn, mns_events).await,
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
    let mut rfcomm = crate::rfcomm::connect(bt_addr, channel).await?;
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
) -> Result<(), TransportError> {
    // Hub accepts the spoke-initiated stream. _recv dropped — spoke sends nothing.
    let (mut send, _recv) = conn.accept_bi().await.map_err(iroh_err)?;
    loop {
        match events.recv().await {
            Ok(payload) => {
                let len = u32::try_from(payload.len())
                    .map_err(|_| TransportError::External("MNS payload exceeds 4 GiB".into()))?;
                send.write_all(&len.to_be_bytes()).await.map_err(iroh_err)?;
                send.write_all(&payload).await.map_err(iroh_err)?;
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("MNS broadcast lagged; {n} events dropped");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
    Ok(())
}
