//! Integration tests for PBAP session lifecycle (connect / disconnect).

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_pbap::{client::PbapClient, PbapError};

const CONNECT_REQ: &[u8] = include_bytes!("fixtures/pbap_connect_req.bin");
const CONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_connect_rsp.bin");
const DISCONNECT_RSP: &[u8] = include_bytes!("fixtures/pbap_disconnect_rsp.bin");

#[tokio::test]
async fn connect_ok() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let req =
                srv.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)?;
            assert_eq!(&req[..], CONNECT_REQ);
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        PbapClient::connect(client_io),
    );
    server_result?;
    client_result?;
    Ok(())
}

#[tokio::test]
async fn connect_rejected() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xC3, 0x00, 0x03]))
                .await
                .map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        PbapClient::connect(client_io),
    );
    server_result?;
    assert!(matches!(client_result, Err(PbapError::Obex(_))));
    Ok(())
}

#[tokio::test]
async fn disconnect_ok() -> Result<(), PbapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(PbapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(DISCONNECT_RSP)).await.map_err(PbapError::Transport)?;
            Ok::<(), PbapError>(())
        },
        async {
            let client = PbapClient::connect(client_io).await?;
            client.disconnect().await
        },
    );
    server_result?;
    client_result
}
