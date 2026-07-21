//! Drives a fake PBAP server over a `tokio::io::duplex` pair — same harness shape as
//! `imsg-pbap`'s own integration tests, but exercised through `sync_contacts` against a real
//! (temp-dir) `Store`.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use secrecy::SecretBox;
use store::Store;

use super::sync_contacts;

// OBEX CONNECT OK response with a ConnectionId header (same fixture bytes as imsg-pbap's
// `tests/fixtures/pbap_connect_rsp.bin`).
const CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// Encodes a single-response OBEX OK packet carrying `body` as `EndOfBody`.
fn ok_body_packet(body: &[u8]) -> anyhow::Result<Bytes> {
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()?)
}

#[tokio::test]
async fn sync_contacts_upserts_every_phone_number() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let (client_io, server_io) = tokio::io::duplex(4096);
    let body = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\n\
TEL:+15551110000\r\nTEL:+15551110001\r\nEND:VCARD\r\n";

    let (server_result, synced) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(ok_body_packet(body)?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(synced?, 2);

    assert_eq!(db.contact_name("+15551110000").await?.as_deref(), Some("Alice"));
    assert_eq!(db.contact_name("+15551110001").await?.as_deref(), Some("Alice"));
    Ok(())
}

#[tokio::test]
async fn sync_contacts_skips_contacts_with_no_phone_numbers() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let (client_io, server_io) = tokio::io::duplex(4096);
    let body = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:NoPhone\r\nEND:VCARD\r\n";

    let (server_result, synced) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(ok_body_packet(body)?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(synced?, 0);
    Ok(())
}

#[tokio::test]
async fn sync_contacts_overwrites_previously_cached_name() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contact("+15551110000", Some("Old Name")).await?;
    let (client_io, server_io) = tokio::io::duplex(4096);
    let body = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:New Name\r\nTEL:+15551110000\r\nEND:VCARD\r\n";

    let (server_result, synced) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(ok_body_packet(body)?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(synced?, 1);

    assert_eq!(db.contact_name("+15551110000").await?.as_deref(), Some("New Name"));
    Ok(())
}
