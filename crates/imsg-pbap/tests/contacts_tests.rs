//! Integration tests for PBAP contact operations.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, normalize_number, phonebook::PhonebookPath, PbapError};

#[test]
fn normalize_us_10digit() {
    assert_eq!(normalize_number("5551234567"), "+15551234567");
}

#[test]
fn normalize_us_formatted() {
    assert_eq!(normalize_number("(555) 123-4567"), "+15551234567");
}

#[test]
fn normalize_us_11digit_with_1() {
    assert_eq!(normalize_number("15551234567"), "+15551234567");
}

#[test]
fn normalize_already_e164() {
    assert_eq!(normalize_number("+15551234567"), "+15551234567");
}

#[test]
fn normalize_non_us_passthrough() {
    assert_eq!(normalize_number("+447911123456"), "+447911123456");
}

#[test]
fn normalize_short_passthrough() {
    assert_eq!(normalize_number("555"), "555");
}

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const BARE_CONTINUE: &[u8] = &[0x90, 0x00, 0x03];

// vCard-listing OK packets (a0 .. / 49 ..) wrapping the XML body.
const LIST_ONE_RSP: &[u8] = b"\xa0\x00\x49\x49\x00\x46<vCard-listing>\
<card handle=\"41.vcf\" name=\"alice\"/></vCard-listing>";
const LIST_OWNER_RSP: &[u8] = b"\xa0\x00\x4c\x49\x00\x49<vCard-listing>\
<card handle=\"0.vcf\" name=\"My Number\"/></vCard-listing>";

// Single-vCard OK packets (a0 00 47 / 49 00 44) wrapping a 65-byte vCard body.
const VCARD_MATCH_RSP: &[u8] = b"\xa0\x00\x47\x49\x00\x44\
BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nTEL:+15551234567\r\nEND:VCARD\r\n";
const VCARD_NOMATCH_RSP: &[u8] = b"\xa0\x00\x47\x49\x00\x44\
BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Carol\r\nTEL:+19998887777\r\nEND:VCARD\r\n";

#[tokio::test]
async fn find_by_number_returns_contact() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            // list(Pb)
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_ONE_RSP)).await.map_err(PbapError::Transport)?;
            // pull(Pb, "41.vcf")
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(VCARD_MATCH_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.find_by_number(PhonebookPath::Pb, "(555) 123-4567").await
        },
    );
    server_result?;
    let found = client_result?;
    let contact = found.ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(contact.display_name.as_deref(), Some("Alice"));
    Ok(())
}

#[tokio::test]
async fn find_by_number_not_found() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            // list(Pb)
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_ONE_RSP)).await.map_err(PbapError::Transport)?;
            // pull(Pb, "41.vcf") — TEL does not match the query
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(VCARD_NOMATCH_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.find_by_number(PhonebookPath::Pb, "+15551234567").await
        },
    );
    server_result?;
    assert!(client_result?.is_none());
    Ok(())
}

#[tokio::test]
async fn find_by_number_skips_owner_card() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            // list(Pb) returns only the owner card; find_by_number must not issue a pull.
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_OWNER_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.find_by_number(PhonebookPath::Pb, "+15551234567").await
        },
    );
    server_result?;
    assert!(client_result?.is_none());
    Ok(())
}

#[tokio::test]
async fn find_by_number_respects_supplied_path() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            // list(Ich)
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_ONE_RSP)).await.map_err(PbapError::Transport)?;
            // pull(Ich, "41.vcf")
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(VCARD_MATCH_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.find_by_number(PhonebookPath::Ich, "(555) 123-4567").await
        },
    );
    server_result?;
    let contact = client_result?.ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(contact.display_name.as_deref(), Some("Alice"));
    Ok(())
}
