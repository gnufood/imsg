//! Unit tests for [`super::handle_mns_event`] against a fake `MapClient` and an in-memory store.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use map_core::mns_event::parse_event_report;
use map_core::BMessage;
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use secrecy::SecretBox;
use store::OutgoingStatus;
use tokio::io::{duplex, DuplexStream};

use super::*;

const CONNECT_RSP: &[u8] = include_bytes!("../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const OK_RSP: &[u8] = &[0xA0, 0x00, 0x03];

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// A message row builder for tests; only the fields each test varies need overriding.
fn sample_message(direction: Direction, outgoing_status: Option<OutgoingStatus>) -> NewMessage {
    let folder =
        if direction == Direction::Sent { "telecom/msg/sent" } else { "telecom/msg/inbox" };
    NewMessage {
        map_handle: "H1".to_owned(),
        timestamp_ms: 0,
        folder: folder.to_owned(),
        direction,
        address: "+1".to_owned(),
        status: STATUS_READ,
        synced_at: 0,
        text: "hi".to_owned(),
        outgoing_status,
    }
}

/// A connected client whose fake server only answers CONNECT — sufficient for every event type
/// except `NewMessage`, the only branch that issues further MAP requests.
async fn fake_client() -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
    });
    Ok(MapClient::connect(client_io).await?)
}

/// A connected client whose fake server also answers the 3-segment `set_folder` SETPATH
/// sequence and a `GetMessage` GET, replying with `wire` as the bMessage body.
async fn fake_client_with_message(wire: String) -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
        for _ in 0..3 {
            let _ = srv.next().await;
            let _ = srv.send(Bytes::from_static(OK_RSP)).await;
        }
        let _ = srv.next().await;
        if let Ok(pkt) = (Packet {
            opcode: OpCode::Ok,
            extra: PacketExtra::None,
            headers: vec![Header::EndOfBody(Bytes::from(wire.into_bytes()))],
        })
        .encode()
        {
            let _ = srv.send(pkt).await;
        }
    });
    Ok(MapClient::connect(client_io).await?)
}

fn event(xml: &str) -> anyhow::Result<MnsEvent> {
    Ok(parse_event_report(xml.as_bytes())?)
}

#[tokio::test]
async fn new_message_upserts_fetched_body() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let wire = BMessage::outbound_sms("+15550002", "hi")
        .encode()
        .replace("FOLDER:telecom/msg/outbox", "FOLDER:telecom/msg/inbox")
        .replace("STATUS:UNREAD", "STATUS:READ")
        .replace("TEL:\r\n", "TEL:+15550001\r\n");
    let mut client = fake_client_with_message(wire).await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='NewMessage' handle='H1' folder='telecom/msg/inbox'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row not upserted"))?;
    assert_eq!(row.text, "hi");
    assert_eq!(row.address, "+15550001");
    Ok(())
}

/// Regression: `imsg-session/src/test_support.rs::NEW_MESSAGE_XML` models a real device's
/// uppercase folder path (`TELECOM/MSG/INBOX`); `parse_folder` must match it case-insensitively
/// or `NewMessage` events are silently dropped on real hardware.
#[tokio::test]
async fn new_message_matches_uppercase_device_folder_path() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let wire = BMessage::outbound_sms("+15550002", "hi")
        .encode()
        .replace("FOLDER:telecom/msg/outbox", "FOLDER:telecom/msg/inbox")
        .replace("STATUS:UNREAD", "STATUS:READ")
        .replace("TEL:\r\n", "TEL:+15550001\r\n");
    let mut client = fake_client_with_message(wire).await?;
    let ev = parse_event_report(crate::test_support::NEW_MESSAGE_XML)?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row = store
        .get_by_handle("ABC123")
        .await?
        .ok_or_else(|| anyhow::anyhow!("row not upserted — uppercase folder path was dropped"))?;
    assert_eq!(row.text, "hi");
    Ok(())
}

#[tokio::test]
async fn message_deleted_removes_row() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.upsert(sample_message(Direction::Received, None)).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='MessageDeleted' handle='H1' folder='telecom/msg/inbox'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    assert!(store.get_by_handle("H1").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn message_shift_updates_folder() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.upsert(sample_message(Direction::Received, None)).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='MessageShift' handle='H1' folder='telecom/msg/deleted' \
         old_folder='telecom/msg/inbox'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.folder, "telecom/msg/deleted");
    Ok(())
}

#[tokio::test]
async fn read_status_changed_marks_read() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let mut msg = sample_message(Direction::Received, None);
    msg.status = store::STATUS_UNREAD;
    store.upsert(msg).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='ReadStatusChanged' handle='H1' folder='telecom/msg/inbox'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.status, STATUS_READ);
    Ok(())
}

/// `DeliverySuccess` must update `outgoing_status`, not the unrelated read/unread flag.
#[tokio::test]
async fn delivery_success_confirms_outgoing_status() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.upsert(sample_message(Direction::Sent, Some(OutgoingStatus::SentUnconfirmed))).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='DeliverySuccess' handle='H1' folder='telecom/msg/sent'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.outgoing_status, Some(OutgoingStatus::SentConfirmed));
    Ok(())
}

#[tokio::test]
async fn sending_success_confirms_outgoing_status() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.upsert(sample_message(Direction::Sent, Some(OutgoingStatus::SentUnconfirmed))).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='SendingSuccess' handle='H1' folder='telecom/msg/sent'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.outgoing_status, Some(OutgoingStatus::SentConfirmed));
    Ok(())
}

#[tokio::test]
async fn delivery_failure_marks_failed_permanent() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.upsert(sample_message(Direction::Sent, Some(OutgoingStatus::SentUnconfirmed))).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='DeliveryFailure' handle='H1' folder='telecom/msg/sent'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.outgoing_status, Some(OutgoingStatus::FailedPermanent));
    Ok(())
}

#[tokio::test]
async fn sending_failure_marks_failed_permanent() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store.upsert(sample_message(Direction::Sent, Some(OutgoingStatus::SentUnconfirmed))).await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='SendingFailure' handle='H1' folder='telecom/msg/sent'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.outgoing_status, Some(OutgoingStatus::FailedPermanent));
    Ok(())
}

#[tokio::test]
async fn memory_full_is_a_no_op() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let mut client = fake_client().await?;
    let ev =
        event("<MAP-event-report version='1.0'><event type='MemoryFull'/></MAP-event-report>")?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    Ok(())
}

#[tokio::test]
async fn memory_available_is_a_no_op() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let mut client = fake_client().await?;
    let ev = event(
        "<MAP-event-report version='1.0'><event type='MemoryAvailable'/></MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    Ok(())
}
