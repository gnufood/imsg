use iroh::endpoint::presets;
use obex_core::{wrap, ObexTransport, TransportError};

use super::{
    iroh_err, Endpoint, EndpointAddr, HubRecvStream, SpokeStream, MAP_ALPN, MNS_ALPN, PBAP_ALPN,
};

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
