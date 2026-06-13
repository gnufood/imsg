//! Integration tests for MAP push-message and set-message-status operations.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_map::{client::MapClient, folders::Folder, MapError, MessageStatus};

const SET_MESSAGE_STATUS_READ_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/put_set_message_status_read_rsp.bin");
const CONNECT_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin");
const TELECOM_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_telecom_rsp.bin");
const MSG_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_msg_rsp.bin");
const INBOX_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_inbox_rsp.bin");
const OUTBOX_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_outbox_rsp.bin");
const PUSH_MESSAGE_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/put_push_message_rsp.bin");
// OBEX OK (0xA0), length 3 — no headers (used to exercise MissingHandle)
const OK_NO_HEADERS_RSP: &[u8] = &[0xA0, 0x00, 0x03];

#[tokio::test]
async fn push_message_returns_handle() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            for rsp in [TELECOM_RSP, MSG_RSP, OUTBOX_RSP] {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(rsp)).await.map_err(MapError::Transport)?;
            }
            let _ = srv.next().await;
            srv.send(Bytes::from_static(PUSH_MESSAGE_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Outbox).await?;
            let handle = client.push_message("+14085551234", "hello").await?;
            assert_eq!(handle, "B997E950DE344EE");
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn push_message_server_error() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xC4, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.push_message("+14085551234", "hello").await;
            assert!(matches!(result, Err(MapError::ServerError(0xC4))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn push_message_invalid_phone_returns_error() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.push_message("+1408\r\n5551234", "hello").await;
            assert!(matches!(result, Err(MapError::InvalidInput(_))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn push_message_missing_handle() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(OK_NO_HEADERS_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.push_message("+14085551234", "hello").await;
            assert!(matches!(result, Err(MapError::MissingHandle)));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_message_status_mark_read() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            for rsp in [TELECOM_RSP, MSG_RSP, INBOX_RSP] {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(rsp)).await.map_err(MapError::Transport)?;
            }
            let _ = srv.next().await;
            srv.send(Bytes::from_static(SET_MESSAGE_STATUS_READ_RSP))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            client.set_message_status_read("AA5E910A67A3416", MessageStatus::Read).await
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_message_status_mark_deleted() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xA0, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_message_status_deleted("AA5E910A67A3416", true).await
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_message_status_server_error() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xC3, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result =
                client.set_message_status_read("AA5E910A67A3416", MessageStatus::Read).await;
            assert!(matches!(result, Err(MapError::ServerError(0xC3))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_message_status_empty_handle() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.set_message_status_read("", MessageStatus::Read).await;
            assert!(matches!(result, Err(MapError::InvalidInput(_))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}
