//! Integration tests for `list_message_folders` — telecom/msg navigation then listing.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_map::{client::MapClient, FolderEntry, MapError};

const CONNECT_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin");
const TELECOM_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_telecom_rsp.bin");
const MSG_RSP: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/setpath_msg_rsp.bin");
const GET_FOLDER_LISTING_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/get_folder_listing_000_rsp.bin");

#[tokio::test]
async fn list_message_folders_navigates_telecom_msg_then_lists() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;

            // Exactly two SETPATHs (telecom, then msg) — and no leaf segment after.
            let first = srv.next().await.ok_or(MapError::UnexpectedEof)??;
            assert!(
                contains(&first, b"\x00t\x00e\x00l\x00e\x00c\x00o\x00m"),
                "first SETPATH is telecom"
            );
            srv.send(Bytes::from_static(TELECOM_RSP)).await.map_err(MapError::Transport)?;

            let second = srv.next().await.ok_or(MapError::UnexpectedEof)??;
            assert!(contains(&second, b"\x00m\x00s\x00g"), "second SETPATH is msg");
            srv.send(Bytes::from_static(MSG_RSP)).await.map_err(MapError::Transport)?;

            // Next op must be the listing GET, not a third SETPATH into a leaf folder.
            let third = srv.next().await.ok_or(MapError::UnexpectedEof)??;
            assert_eq!(third.first().copied(), Some(0x83), "third op is GET_FINAL, not SETPATH");
            srv.send(Bytes::from_static(GET_FOLDER_LISTING_RSP))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let listing = client.list_message_folders().await?;
            let names: Vec<&str> = listing.folders().iter().map(FolderEntry::name).collect();
            assert_eq!(names, ["inbox", "sent", "outbox", "deleted"]);
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

#[tokio::test]
async fn list_message_folders_server_error_on_setpath() -> Result<(), MapError> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, client_result) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            srv.send(Bytes::from_static(TELECOM_RSP)).await.map_err(MapError::Transport)?;
            let _ = srv.next().await;
            // FORBIDDEN on the msg SETPATH.
            srv.send(Bytes::copy_from_slice(&[0xC3, 0x00, 0x03]))
                .await
                .map_err(MapError::Transport)?;
            Ok::<(), MapError>(())
        },
        async {
            let mut client = MapClient::connect(client_io).await?;
            let result = client.list_message_folders().await;
            assert!(matches!(result, Err(MapError::ServerError(0xC3))));
            Ok::<(), MapError>(())
        },
    );
    server_result?;
    client_result
}

/// True if `needle` appears as a contiguous byte run anywhere in `haystack`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}
