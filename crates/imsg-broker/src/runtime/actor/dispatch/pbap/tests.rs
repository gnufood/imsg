//! Unit tests for [`super::do_sync_contacts`].

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use pbap_core::client::PbapClient;
use secrecy::SecretBox;
use store::Store;
use tokio::io::{duplex, DuplexStream};

use super::*;

// OBEX CONNECT OK response with a ConnectionId header (same fixture bytes as imsg-pbap's
// `tests/fixtures/pbap_connect_rsp.bin`).
const PBAP_CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// Encodes a single-response OBEX OK packet carrying `body` as `EndOfBody`.
fn ok_body_packet(body: &[u8]) -> anyhow::Result<Bytes> {
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()?)
}

/// A connected PBAP client whose fake server answers CONNECT then one `pull_all` with `body`.
async fn fake_pbap_client(body: &'static [u8]) -> anyhow::Result<PbapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(PBAP_CONNECT_RSP)).await;
        let _ = srv.next().await;
        if let Ok(pkt) = ok_body_packet(body) {
            let _ = srv.send(pkt).await;
        }
    });
    Ok(PbapClient::connect(client_io).await?)
}

/// In-memory `Store` (temp-dir `SQLite`) plus the dir guard.
async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// A full sync writes every phone number on the pulled contact into the store cache and reports
/// the row count.
#[tokio::test]
async fn do_sync_contacts_upserts_and_reports_count() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let body = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nTEL:+15551110000\r\nEND:VCARD\r\n";
    let mut pbap = fake_pbap_client(body).await?;

    let resp = do_sync_contacts(&mut pbap, &store).await?;

    assert!(matches!(resp, BrokerResponse::ContactsSynced { count: 1 }));
    assert_eq!(store.contact_name("+15551110000").await?.as_deref(), Some("Alice"));
    Ok(())
}
