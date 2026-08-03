//! `list` (`ListvCardObjects`) fixture tests: windowing and the no-window omitted-`AppParams` case.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, phonebook::PhonebookPath, PbapError};

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const LIST_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_list_rsp.bin");

// OK packet + EndOfBody header (a0 00 6b / 49 00 68) wrapping a 101-byte vCard-listing XML body.
const LIST_BODY_RSP: &[u8] = b"\xa0\x00\x6b\x49\x00\x68<vCard-listing>\
<card handle=\"41.vcf\" name=\"alice\"/>\
<card handle=\"42.vcf\" name=\"bob\"/>\
</vCard-listing>";

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
            client.list(PhonebookPath::Pb, None, 0).await
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
async fn list_no_window_omits_app_params() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(!req.windows(1).any(|w| w == [0x4C]));
            srv.send(Bytes::from_static(LIST_BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.list(PhonebookPath::Pb, None, 0).await
        },
    );
    server_result?;
    client_result?;
    Ok(())
}

#[tokio::test]
async fn list_sends_max_list_count_and_offset() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(4).any(|w| w == [0x04, 0x02, 0x00, 0x0A]));
            assert!(req.windows(4).any(|w| w == [0x05, 0x02, 0x00, 0x1E]));
            srv.send(Bytes::from_static(LIST_BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.list(PhonebookPath::Pb, Some(10), 30).await
        },
    );
    server_result?;
    client_result?;
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
            client.list(PhonebookPath::Pb, None, 0).await
        },
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::ServerError(0xC4))));
    Ok(())
}
