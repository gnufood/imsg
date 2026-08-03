//! `pull_all` fixture tests: windowing (limit/offset), non-Pb phonebook paths, and CONTINUE+body
//! accumulation.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, phonebook::PhonebookPath, PbapError};

const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const PULL_ALL_BARE_CONTINUE: &[u8] = include_bytes!("fixtures/pbap_pull_all_rsp.bin");

// OK packet with EndOfBody = one minimal vCard (57 bytes of body).
// Packet: [0xA0, 0x00, 0x3F] + EndOfBody header [0x49, 0x00, 0x3C] + 57 vCard bytes.
const PULL_ALL_BODY_RSP: &[u8] = &[
    0xA0, 0x00, 0x3F, 0x49, 0x00, 0x3C, b'B', b'E', b'G', b'I', b'N', b':', b'V', b'C', b'A', b'R',
    b'D', b'\r', b'\n', b'V', b'E', b'R', b'S', b'I', b'O', b'N', b':', b'3', b'.', b'0', b'\r',
    b'\n', b'F', b'N', b':', b'T', b'e', b's', b't', b'\r', b'\n', b'T', b'E', b'L', b':', b'+',
    b'1', b'1', b'1', b'1', b'\r', b'\n', b'E', b'N', b'D', b':', b'V', b'C', b'A', b'R', b'D',
    b'\r', b'\n',
];

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
            assert!(req.windows(3).any(|w| w == [0x07, 0x01, 0x01]));
            assert!(req.windows(4).any(|w| w == [0x04, 0x02, 0xff, 0xff]));
            assert!(req
                .windows(10)
                .any(|w| w == [0x06, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x83]));
            srv.send(Bytes::from_static(PULL_ALL_BARE_CONTINUE))
                .await
                .map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(PULL_ALL_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull_all(PhonebookPath::Pb, None, 0).await
        },
    );
    server_result?;
    let contacts = client_result?;
    assert_eq!(contacts.len(), 1);
    let first = contacts.first().ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(first.display_name.as_deref(), Some("Test"));
    let phone = first.phones().first().ok_or(PbapError::UnexpectedEof)?;
    assert_eq!(phone.raw(), "+1111");
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
            client.pull_all(PhonebookPath::Pb, None, 0).await
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
            client.pull_all(PhonebookPath::Ich, None, 0).await
        },
    );
    server_result?;
    client_result?;
    Ok(())
}

#[tokio::test]
async fn pull_all_sends_max_list_count_and_offset() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert!(req.windows(4).any(|w| w == [0x04, 0x02, 0x00, 0x14]));
            assert!(req.windows(4).any(|w| w == [0x05, 0x02, 0x00, 0x28]));
            srv.send(Bytes::from_static(PULL_ALL_BARE_CONTINUE))
                .await
                .map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(PULL_ALL_BODY_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            client.pull_all(PhonebookPath::Pb, Some(20), 40).await
        },
    );
    server_result?;
    client_result?;
    Ok(())
}
