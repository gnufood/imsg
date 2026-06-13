//! Integration tests for MAP list-messages and get-message operations.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_map::{
    client::MapClient,
    folders::Folder,
    messages::{ListMessagesFilter, ReadStatus},
    MapError, MessageStatus,
};

const CONNECT_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin");
const TELECOM_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_telecom_rsp.bin");
const MSG_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_msg_rsp.bin");
const INBOX_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_inbox_rsp.bin");
const LIST_MESSAGES_INBOX_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/list_messages_inbox_rsp.bin");
const LIST_MESSAGES_UNREAD_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/list_messages_unread_rsp.bin");
const GET_MESSAGE_000_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/get_message_000_rsp.bin");
// OBEX OK (0xA0), EndOfBody (0x49), 2-byte payload [0xFF, 0xFE] — invalid UTF-8
const BAD_UTF8_RSP: &[u8] = &[0xA0, 0x00, 0x08, 0x49, 0x00, 0x05, 0xFF, 0xFE];

#[tokio::test]
async fn list_messages_inbox_returns_entries() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(8192);

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
            srv.send(Bytes::from_static(LIST_MESSAGES_INBOX_RSP))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let entries = client.list_messages(&ListMessagesFilter::default()).await?;
            assert_eq!(entries.len(), 10);
            let first = entries.first().ok_or(MapError::UnexpectedEof)?;
            assert_eq!(first.handle, "AA5E910A67A3416");
            assert!(!first.read);
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn list_messages_unread_filter_returns_unread_only() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(8192);
    let filter = ListMessagesFilter {
        read_status: Some(ReadStatus::Unread),
        ..ListMessagesFilter::default()
    };

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
            srv.send(Bytes::from_static(LIST_MESSAGES_UNREAD_RSP))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let entries = client.list_messages(&filter).await?;
            assert!(!entries.is_empty());
            assert!(entries.iter().all(|e| !e.read));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_message_returns_bmessage() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(8192);

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
            srv.send(Bytes::from_static(GET_MESSAGE_000_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let msg = client.get_message("AA5E910A67A3416").await?;
            assert_eq!(*msg.status(), MessageStatus::Unread);
            assert_eq!(msg.folder(), "telecom/msg/inbox");
            assert!(msg.originator().is_some_and(|o| o.tel == "5550001001"));
            assert!(msg.envelope().body.text.contains("Synthetic fixture"));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_message_invalid_utf8_body_returns_invalid_encoding() -> Result<(), MapError> {
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
            srv.send(Bytes::from_static(BAD_UTF8_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let result = client.get_message("AA5E910A67A3416").await;
            assert!(matches!(result, Err(MapError::InvalidEncoding)));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_message_returns_server_error_on_rejection() -> Result<(), MapError> {
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
            srv.send(Bytes::copy_from_slice(&[0xC4, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let result = client.get_message("AA5E910A67A3416").await;
            assert!(matches!(result, Err(MapError::ServerError(0xC4))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_message_empty_handle_is_rejected() -> Result<(), MapError> {
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
            let result = client.get_message("").await;
            assert!(matches!(result, Err(MapError::InvalidInput(_))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_message_crlf_handle_is_rejected() -> Result<(), MapError> {
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
            let result = client.get_message("AA5E\r\n910A").await;
            assert!(matches!(result, Err(MapError::InvalidInput(_))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn list_messages_with_originator_filter() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(8192);
    let filter = ListMessagesFilter {
        originating_address: Some("5550001001".to_owned()),
        ..ListMessagesFilter::default()
    };

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            for rsp in [TELECOM_RSP, MSG_RSP, INBOX_RSP] {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(rsp)).await.map_err(MapError::Transport)?;
            }
            let req =
                srv.next().await.ok_or(MapError::UnexpectedEof)?.map_err(MapError::Transport)?;
            let tlv: &[u8] = b"\x08\x0b5550001001\x00";
            assert!(req.windows(tlv.len()).any(|w| w == tlv));
            srv.send(Bytes::from_static(LIST_MESSAGES_INBOX_RSP))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let entries = client.list_messages(&filter).await?;
            assert!(!entries.is_empty());
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}
