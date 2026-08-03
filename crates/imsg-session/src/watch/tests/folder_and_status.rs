use map_core::BMessage;
use store::STATUS_READ;

use super::{
    event, fake_client, fake_client_with_message, fake_store, handle_mns_event, sample_message,
    Direction,
};

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

/// `ReadStatusChanged` carries no directionality in MAP itself — the fix re-fetches the
/// message and trusts its actual `STATUS`, rather than assuming the event always means
/// "became read".
#[tokio::test]
async fn read_status_changed_marks_read() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let mut msg = sample_message(Direction::Received, None);
    msg.status = store::STATUS_UNREAD;
    store.upsert(msg).await?;
    let wire = BMessage::outbound_sms("+15550002", "hi")
        .encode()
        .replace("FOLDER:telecom/msg/outbox", "FOLDER:telecom/msg/inbox")
        .replace("STATUS:UNREAD", "STATUS:READ")
        .replace("TEL:\r\n", "TEL:+15550001\r\n");
    let mut client = fake_client_with_message(wire).await?;
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

/// Regression: `ReadStatusChanged` previously hardcoded `STATUS_READ` regardless of the
/// message's true status. A message re-marked unread on the device must be reflected as
/// unread locally too, not silently forced to read.
#[tokio::test]
async fn read_status_changed_reflects_actual_unread_status() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let mut msg = sample_message(Direction::Received, None);
    msg.status = STATUS_READ;
    store.upsert(msg).await?;
    let wire = BMessage::outbound_sms("+15550002", "hi")
        .encode()
        .replace("FOLDER:telecom/msg/outbox", "FOLDER:telecom/msg/inbox")
        .replace("TEL:\r\n", "TEL:+15550001\r\n");
    let mut client = fake_client_with_message(wire).await?;
    let ev = event(
        "<MAP-event-report version='1.0'>\
         <event type='ReadStatusChanged' handle='H1' folder='telecom/msg/inbox'/>\
         </MAP-event-report>",
    )?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row =
        store.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row unexpectedly gone"))?;
    assert_eq!(row.status, store::STATUS_UNREAD);
    Ok(())
}
