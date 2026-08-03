//! `search` (`ListvCardObjects` with `SearchAttribute`/`SearchValue`) fixture tests, including
//! the CR/LF search-value rejection.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{
    client::PbapClient,
    phonebook::{PhonebookPath, SearchAttribute},
    PbapError,
};

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const LIST_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_list_rsp.bin");

// OK packet + EndOfBody header (a0 00 6b / 49 00 68) wrapping a 101-byte vCard-listing XML body.
const LIST_BODY_RSP: &[u8] = b"\xa0\x00\x6b\x49\x00\x68<vCard-listing>\
<card handle=\"41.vcf\" name=\"alice\"/>\
<card handle=\"42.vcf\" name=\"bob\"/>\
</vCard-listing>";

#[tokio::test]
async fn search_sends_attribute_and_value() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(18).any(|w| w == b"x-bt/vcard-listing"));
            assert!(req.windows(3).any(|w| w == [0x03, 0x01, 0x01]));
            assert!(req.windows(11).any(|w| w == b"\x02\x09+15550001"));
            srv.send(Bytes::from_static(LIST_BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.search(PhonebookPath::Pb, SearchAttribute::Number, "+15550001", None, 0).await
        },
    );
    server_result?;
    let entries = client_result?;
    assert_eq!(entries.len(), 2);
    Ok(())
}

#[tokio::test]
async fn search_sends_window_when_given() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(4).any(|w| w == [0x04, 0x02, 0x00, 0x05]));
            srv.send(Bytes::from_static(LIST_BARE_CONTINUE)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(LIST_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.search(PhonebookPath::Pb, SearchAttribute::Name, "alice", Some(5), 0).await
        },
    );
    server_result?;
    client_result?;
    Ok(())
}

#[tokio::test]
async fn search_rejects_value_with_cr_or_lf() -> Result<(), PbapError> {
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
            client.search(PhonebookPath::Pb, SearchAttribute::Name, "a\r\nb", None, 0).await
        },
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::InvalidInput(_))));
    Ok(())
}
