//! Integration tests for PBAP phonebook pull and vCard-listing operations.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, phonebook::PhonebookPath, PbapError};

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const PULL_ALL_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_pull_all_rsp.bin");
const LIST_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_list_rsp.bin");
const PULL_ENTRY_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_pull_entry_rsp.bin");

// OK packet with EndOfBody = one minimal vCard (57 bytes of body).
// Packet: [0xA0, 0x00, 0x3F] + EndOfBody header [0x49, 0x00, 0x3C] + 57 vCard bytes.
const PULL_ALL_BODY_RSP: &[u8] = &[
    0xA0, 0x00, 0x3F, 0x49, 0x00, 0x3C, b'B', b'E', b'G', b'I', b'N', b':', b'V', b'C', b'A', b'R',
    b'D', b'\r', b'\n', b'V', b'E', b'R', b'S', b'I', b'O', b'N', b':', b'3', b'.', b'0', b'\r',
    b'\n', b'F', b'N', b':', b'T', b'e', b's', b't', b'\r', b'\n', b'T', b'E', b'L', b':', b'+',
    b'1', b'1', b'1', b'1', b'\r', b'\n', b'E', b'N', b'D', b':', b'V', b'C', b'A', b'R', b'D',
    b'\r', b'\n',
];

// OK packet + EndOfBody header (a0 00 6b / 49 00 68) wrapping a 101-byte vCard-listing XML body.
const LIST_BODY_RSP: &[u8] = b"\xa0\x00\x6b\x49\x00\x68<vCard-listing>\
<card handle=\"41.vcf\" name=\"alice\"/>\
<card handle=\"42.vcf\" name=\"bob\"/>\
</vCard-listing>";

// OK packet + EndOfBody header (a0 00 45 / 49 00 42) wrapping a 63-byte single vCard body.
const PULL_ENTRY_BODY_RSP: &[u8] = b"\xa0\x00\x45\x49\x00\x42\
BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Bob\r\nTEL:+15551234567\r\nEND:VCARD\r\n";

// ── pull_all tests (moved from pbap_tests.rs) ─────────────────────────────────

#[tokio::test]
async fn pull_all_pb_returns_contacts() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(14).any(|w| w == b"x-bt/phonebook"));
            assert!(req.windows(7).any(|w| w == [0x07, 0x01, 0x01, 0x04, 0x02, 0xff, 0xff]));
            srv.send(Bytes::from_static(PULL_ALL_BARE_CONTINUE))
                .await
                .map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(PULL_ALL_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull_all(PhonebookPath::Pb).await
        },
    );
    server_result?;
    let contacts = client_result?;
    assert_eq!(contacts.len(), 1);
    let first = contacts.first().ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(first.display_name.as_deref(), Some("Test"));
    assert_eq!(first.phones(), ["+1111"]);
    Ok(())
}

#[tokio::test]
async fn pull_all_server_error() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xC4, 0x00, 0x03]))
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull_all(PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::ServerError(0xC4))));
    Ok(())
}

#[tokio::test]
async fn pull_all_ich_uses_correct_name() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(6).any(|w| w == [0x00, 0x69, 0x00, 0x63, 0x00, 0x68]));
            srv.send(Bytes::from_static(PULL_ALL_BARE_CONTINUE))
                .await
                .map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(PULL_ALL_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull_all(PhonebookPath::Ich).await
        },
    );
    server_result?;
    client_result?;
    Ok(())
}

// ── list tests ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_pb_returns_card_entries() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(18).any(|w| w == b"x-bt/vcard-listing"));
            srv.send(Bytes::from_static(LIST_BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.list(PhonebookPath::Pb).await
        },
    );
    server_result?;
    let entries = client_result?;
    assert_eq!(entries.len(), 2);
    let first = entries.first().ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(first.handle(), "41.vcf");
    assert_eq!(first.name(), Some("alice"));
    Ok(())
}

#[tokio::test]
async fn list_server_error() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xC4, 0x00, 0x03]))
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.list(PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::ServerError(0xC4))));
    Ok(())
}

// ── pull entry tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn pull_entry_returns_contact() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            // Type is the entry type, not the listing type (null terminator distinguishes them).
            assert!(req.windows(11).any(|w| w == b"x-bt/vcard\x00"));
            // Name contains the handle "5.vcf" in UTF-16BE.
            assert!(req
                .windows(10)
                .any(|w| w == [0x00, 0x35, 0x00, 0x2e, 0x00, 0x76, 0x00, 0x63, 0x00, 0x66]));
            srv.send(Bytes::from_static(PULL_ENTRY_BARE_CONTINUE))
                .await
                .map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(PULL_ENTRY_BODY_RSP))
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull(PhonebookPath::Pb, "5.vcf").await
        },
    );
    server_result?;
    let contact = client_result?;
    assert_eq!(contact.display_name.as_deref(), Some("Bob"));
    assert_eq!(contact.phones(), ["+15551234567"]);
    Ok(())
}

#[tokio::test]
async fn pull_entry_server_error() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xC4, 0x00, 0x03]))
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull(PhonebookPath::Pb, "5.vcf").await
        },
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::ServerError(0xC4))));
    Ok(())
}

#[tokio::test]
async fn pull_entry_empty_handle() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull(PhonebookPath::Pb, "").await
        },
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::InvalidInput(_))));
    Ok(())
}
