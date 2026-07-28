//! Unit tests for [`super::do_delete`]'s `Watch` fan-out.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use ipc::EventType;
use secrecy::SecretBox;
use store::Store;
use tokio::io::{duplex, DuplexStream};
use tokio::sync::broadcast;

use super::*;

const CONNECT_RSP: &[u8] =
    include_bytes!("../../../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const TELECOM_RSP: &[u8] =
    include_bytes!("../../../../../../imsg-obex/tests/fixtures/setpath_telecom_rsp.bin");
const MSG_RSP: &[u8] =
    include_bytes!("../../../../../../imsg-obex/tests/fixtures/setpath_msg_rsp.bin");
const INBOX_RSP: &[u8] =
    include_bytes!("../../../../../../imsg-obex/tests/fixtures/setpath_inbox_rsp.bin");
// OBEX OK (0xA0), length 3, no headers — accepted response for the `SetMessageStatus` PUT.
const OK_NO_HEADERS_RSP: &[u8] = &[0xA0, 0x00, 0x03];
const FOLDER_LISTING_RSP: &[u8] =
    include_bytes!("../../../../../../imsg-obex/tests/fixtures/get_folder_listing_000_rsp.bin");

/// In-memory `Store` (temp-dir `SQLite`) plus the dir guard.
async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// A connected client whose fake server answers CONNECT, the three `set_folder(Inbox)` SETPATHs,
/// then the `SetMessageStatus` PUT with a bare OK — the full sequence `do_delete` drives.
async fn fake_client() -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
        for rsp in [TELECOM_RSP, MSG_RSP, INBOX_RSP, OK_NO_HEADERS_RSP] {
            let _ = srv.next().await;
            let _ = srv.send(Bytes::copy_from_slice(rsp)).await;
        }
    });
    Ok(MapClient::connect(client_io).await?)
}

/// A connected client whose fake server answers CONNECT, the `telecom` → `msg` SETPATHs, then
/// the captured folder-listing GET — the sequence `do_live_folders` drives.
async fn fake_folders_client() -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
        for rsp in [TELECOM_RSP, MSG_RSP, FOLDER_LISTING_RSP] {
            let _ = srv.next().await;
            let _ = srv.send(Bytes::copy_from_slice(rsp)).await;
        }
    });
    Ok(MapClient::connect(client_io).await?)
}

/// The device's document order is the contract — the CLI prints the listing as-is, so the
/// mapping to DTOs must not sort or dedupe.
#[tokio::test]
async fn do_live_folders_returns_device_listing_in_order() -> anyhow::Result<()> {
    let mut client = fake_folders_client().await?;

    let resp = do_live_folders(&mut client).await?;

    let BrokerResponse::Folders(rows) = resp else {
        anyhow::bail!("expected Folders, got {resp:?}");
    };
    let names: Vec<String> = rows.into_iter().map(|f| f.name).collect();
    assert_eq!(names, ["inbox", "sent", "outbox", "deleted"]);
    Ok(())
}

/// A successful delete must fan a `MessageDeleted` event out to `Watch` subscribers — the only
/// signal any other client gets, since delete has no MNS-driven confirmation to fall back on.
#[tokio::test]
async fn do_delete_broadcasts_message_deleted_on_success() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let mut client = fake_client().await?;
    let (watch_tx, mut watch_rx) = broadcast::channel(4);

    let resp =
        do_delete(&mut client, &store, "H1".to_owned(), "inbox".to_owned(), &watch_tx).await?;

    assert!(matches!(resp, BrokerResponse::Text(t) if t == "deleted H1"));
    let got = watch_rx.try_recv()?;
    assert_eq!(got.event_type, EventType::MessageDeleted);
    assert_eq!(got.handle.as_deref(), Some("H1"));
    assert_eq!(got.folder.as_deref(), Some("inbox"));
    assert_eq!(got.old_folder, None);
    assert_eq!(got.msg_type, None);
    assert_eq!(got.datetime, None);
    Ok(())
}

/// An unknown folder name is rejected before any MAP/store work — nothing to report, so no
/// broadcast should fire.
#[tokio::test]
async fn do_delete_unknown_folder_does_not_broadcast() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    // An unknown folder name returns before any MAP request beyond CONNECT is sent.
    let mut client = fake_client().await?;
    let (watch_tx, mut watch_rx) = broadcast::channel(4);

    let resp =
        do_delete(&mut client, &store, "H1".to_owned(), "bogus".to_owned(), &watch_tx).await?;

    assert!(matches!(resp, BrokerResponse::Failed(Reason::OperationFailed(_))));
    assert!(watch_rx.try_recv().is_err(), "unknown-folder failure must not broadcast");
    Ok(())
}
