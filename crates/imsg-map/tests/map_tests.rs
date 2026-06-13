//! Integration tests for MAP session, folder navigation, and notification registration.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_map::{client::MapClient, folders::Folder, MapError};

const CONNECT_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin");
const TELECOM_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_telecom_rsp.bin");
const MSG_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_msg_rsp.bin");
const INBOX_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_inbox_rsp.bin");
// OBEX OK response: opcode 0xA0, length 3, no headers.
const SETPATH_OK_RSP: &[u8] = &[0xA0, 0x00, 0x03];
const GET_FOLDER_LISTING_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/get_folder_listing_000_rsp.bin");
// OBEX OK, EndOfBody = b"<folder-listing" (unclosed tag — quick-xml UnexpectedEof)
const BAD_XML_RSP: &[u8] = &[
    0xA0, 0x00, 0x15, // OK, total 21
    0x49, 0x00, 0x12, // EndOfBody, 18 bytes (1+2+15)
    b'<', b'f', b'o', b'l', b'd', b'e', b'r', b'-', b'l', b'i', b's', b't', b'i', b'n', b'g',
];

#[tokio::test]
async fn set_folder_inbox_succeeds() -> Result<(), MapError> {
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
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_folder_returns_server_error_on_rejection() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            // NOT_FOUND: opcode 0xC4, length 3
            srv.send(Bytes::copy_from_slice(&[0xC4, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.set_folder(Folder::Inbox).await;
            assert!(matches!(result, Err(MapError::ServerError(0xC4))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_folder_listing_returns_folders() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(GET_FOLDER_LISTING_RSP))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let listing = client.get_folder_listing().await?;
            let names: Vec<&str> =
                listing.folders().iter().map(imsg_map::FolderEntry::name).collect();
            assert_eq!(names, ["inbox", "sent", "outbox", "deleted"]);
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_folder_listing_server_error() -> Result<(), MapError> {
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
            let result = client.get_folder_listing().await;
            assert!(matches!(result, Err(MapError::ServerError(0xC4))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn get_folder_listing_malformed_xml() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(BAD_XML_RSP)).await.map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.get_folder_listing().await;
            assert!(matches!(result, Err(MapError::FolderListing(_))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_notification_registration_on_succeeds() -> Result<(), MapError> {
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
            client.set_notification_registration(true).await
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_notification_registration_off_succeeds() -> Result<(), MapError> {
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
            client.set_notification_registration(false).await
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_folder_inbox_then_sent_succeeds() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            // inbox: telecom → msg → inbox
            for rsp in [TELECOM_RSP, MSG_RSP, INBOX_RSP] {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(rsp)).await.map_err(MapError::Transport)?;
            }
            // backup × 3 (depth 3 → 0)
            for _ in 0..3 {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(SETPATH_OK_RSP)).await.map_err(MapError::Transport)?;
            }
            // sent: telecom → msg → sent
            for _ in 0..3 {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(SETPATH_OK_RSP)).await.map_err(MapError::Transport)?;
            }
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            client.set_folder(Folder::Sent).await
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_folder_backup_server_error_propagated() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            // inbox navigation succeeds
            for rsp in [TELECOM_RSP, MSG_RSP, INBOX_RSP] {
                let _ = srv.next().await;
                srv.send(Bytes::from_static(rsp)).await.map_err(MapError::Transport)?;
            }
            // first backup rejected: RSP_NOT_IMPLEMENTED 0xD0
            let _ = srv.next().await;
            srv.send(Bytes::copy_from_slice(&[0xD0, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            client.set_folder(Folder::Inbox).await?;
            let result = client.set_folder(Folder::Sent).await;
            assert!(matches!(result, Err(MapError::ServerError(0xD0))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn set_notification_registration_server_error() -> Result<(), MapError> {
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
            let result = client.set_notification_registration(true).await;
            assert!(matches!(result, Err(MapError::ServerError(0xC3))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}
