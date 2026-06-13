//! iroh transport tests — key persistence and loopback hub/spoke OBEX and MNS round-trips.
//!
//! No Bluetooth required: QUIC runs over loopback with the relay disabled. The hub sides are
//! minimal byte-echo or push servers so the spoke's framing decodes exactly what was sent.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_transport::iroh::{
    connect_map_hub, connect_mns_hub, connect_pbap_hub, load_or_create_key, EndpointAddr, MAP_ALPN,
    MNS_ALPN, PBAP_ALPN,
};
use iroh::endpoint::presets;
use iroh::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn key_creates_and_persists() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("nested/hub.key");
    let created = load_or_create_key(&path).await?;
    assert!(path.exists(), "key file should be created");
    let reloaded = load_or_create_key(&path).await?;
    assert_eq!(created.to_bytes(), reloaded.to_bytes(), "reload must return the same key");
    Ok(())
}

#[tokio::test]
async fn key_load_existing() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("hub.key");
    let seed = [7u8; 32];
    std::fs::write(&path, seed)?;
    let key = load_or_create_key(&path).await?;
    assert_eq!(key.to_bytes(), seed, "existing file must be read, not regenerated");
    Ok(())
}

/// Accepts one connection and echoes the first stream's bytes back, holding the connection
/// open until the peer closes so in-flight delivery is not aborted.
async fn echo_once(endpoint: Endpoint) -> TestResult {
    let incoming = endpoint.accept().await.ok_or("no incoming connection")?;
    let conn = incoming.accept()?.await?;
    let (mut send, mut recv) = conn.accept_bi().await?;
    let mut buf = vec![0u8; 4096];
    let n = recv.read(&mut buf).await?.unwrap_or(0);
    send.write_all(buf.get(..n).unwrap_or(&[])).await?;
    send.shutdown().await?;
    conn.closed().await;
    Ok(())
}

/// Accepts one MNS connection, waits for the spoke to open a stream, pushes `payload`
/// length-framed (4-byte BE prefix), and shuts down.
async fn mns_push_once(endpoint: Endpoint, payload: Bytes) -> TestResult {
    let incoming = endpoint.accept().await.ok_or("no incoming connection")?;
    let conn = incoming.accept()?.await?;
    let (mut send, _recv) = conn.accept_bi().await?;
    let len = u32::try_from(payload.len())?;
    send.write_all(&len.to_be_bytes()).await?;
    send.write_all(&payload).await?;
    send.shutdown().await?;
    conn.closed().await;
    Ok(())
}

/// Binds a loopback hub on `alpn`, returning its dialable address plus the live endpoint.
async fn echo_hub(
    alpn: &'static [u8],
) -> Result<(Endpoint, EndpointAddr), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint =
        Endpoint::builder(presets::N0DisableRelay).alpns(vec![alpn.to_vec()]).bind().await?;
    let addr = endpoint
        .bound_sockets()
        .into_iter()
        .fold(EndpointAddr::new(endpoint.addr().id), EndpointAddr::with_ip_addr);
    let server = endpoint.clone();
    tokio::spawn(async move {
        let _ = echo_once(server).await;
    });
    Ok((endpoint, addr))
}

/// Binds a loopback MNS hub that pushes `payload` once to the first spoke.
async fn mns_hub(
    payload: Bytes,
) -> Result<(Endpoint, EndpointAddr), Box<dyn std::error::Error + Send + Sync>> {
    let endpoint =
        Endpoint::builder(presets::N0DisableRelay).alpns(vec![MNS_ALPN.to_vec()]).bind().await?;
    let addr = endpoint
        .bound_sockets()
        .into_iter()
        .fold(EndpointAddr::new(endpoint.addr().id), EndpointAddr::with_ip_addr);
    let server = endpoint.clone();
    tokio::spawn(async move {
        let _ = mns_push_once(server, payload).await;
    });
    Ok((endpoint, addr))
}

async fn roundtrip(alpn: &'static [u8], use_pbap: bool) -> TestResult {
    let (_hub, hub_addr) = echo_hub(alpn).await?;
    let spoke = Endpoint::builder(presets::N0DisableRelay).bind().await?;
    let mut transport = if use_pbap {
        connect_pbap_hub(&spoke, hub_addr).await?
    } else {
        connect_map_hub(&spoke, hub_addr).await?
    };
    let packet =
        Bytes::from_static(include_bytes!("../../imsg-obex/tests/fixtures/connect_req.bin"));
    transport.send(packet.clone()).await?;
    let echoed = transport.next().await.ok_or("stream ended before echo")??;
    assert_eq!(echoed, packet, "hub must echo the framed packet intact");
    Ok(())
}

#[tokio::test]
async fn hub_spoke_map_connect() -> TestResult {
    roundtrip(MAP_ALPN, false).await
}

#[tokio::test]
async fn hub_spoke_pbap_connect() -> TestResult {
    roundtrip(PBAP_ALPN, true).await
}

#[tokio::test]
async fn hub_spoke_mns_subscribe() -> TestResult {
    let payload = Bytes::from_static(b"NewMessage handle=ABC123 folder=TELECOM/MSG/INBOX\n");
    let (_hub, hub_addr) = mns_hub(payload.clone()).await?;
    let spoke = Endpoint::builder(presets::N0DisableRelay).bind().await?;
    let mut recv = connect_mns_hub(&spoke, hub_addr).await?;
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let len = usize::try_from(u32::from_be_bytes(len_buf))?;
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await?;
    assert_eq!(buf, &payload[..], "hub must deliver length-framed MNS payload to spoke");
    Ok(())
}
