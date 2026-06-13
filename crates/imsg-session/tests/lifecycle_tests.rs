//! Integration tests for session lifecycle — `establish_map_session` and `establish_pbap_session`.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_session::{lifecycle, SessionError};

const MAP_CONNECT_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin");

#[tokio::test]
async fn establish_map_session_succeeds() -> Result<(), SessionError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(MAP_CONNECT_RSP)).await.map_err(SessionError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(&[0xA0, 0x00, 0x03]))
                .await
                .map_err(SessionError::Transport)?;
            Ok::<(), SessionError>(())
        },
        lifecycle::establish_map_session(client_io),
    );
    server_result?;
    client_result.map(|_| ())
}

#[tokio::test]
async fn establish_map_session_connect_error() -> Result<(), SessionError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(&[0xC0, 0x00, 0x03]))
                .await
                .map_err(SessionError::Transport)?;
            Ok::<(), SessionError>(())
        },
        lifecycle::establish_map_session(client_io),
    );
    server_result?;
    assert!(matches!(client_result, Err(SessionError::Map(_))));
    Ok(())
}

#[tokio::test]
async fn establish_map_session_notif_reg_error() -> Result<(), SessionError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(MAP_CONNECT_RSP)).await.map_err(SessionError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(&[0xC3, 0x00, 0x03]))
                .await
                .map_err(SessionError::Transport)?;
            Ok::<(), SessionError>(())
        },
        lifecycle::establish_map_session(client_io),
    );
    server_result?;
    assert!(matches!(client_result, Err(SessionError::Map(_))));
    Ok(())
}
