//! `pull` (`PullvCardEntry`) fixture tests, including the empty-handle rejection.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, phonebook::PhonebookPath, PbapError};

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const PULL_ENTRY_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_pull_entry_rsp.bin");

// OK packet + EndOfBody header (a0 00 45 / 49 00 42) wrapping a 63-byte single vCard body.
const PULL_ENTRY_BODY_RSP: &[u8] = b"\xa0\x00\x45\x49\x00\x42\
BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Bob\r\nTEL:+15551234567\r\nEND:VCARD\r\n";

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
            assert!(req
                .windows(10)
                .any(|w| w == [0x06, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x83]));
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
    let phone = contact.phones().first().ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(phone.raw(), "+15551234567");
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
