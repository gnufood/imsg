use map_core::BMessage;

use super::{event, fake_client_with_message, fake_store, handle_mns_event};

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
    let ev = map_core::mns_event::parse_event_report(crate::test_support::NEW_MESSAGE_XML)?;
    handle_mns_event(&ev, &mut client, &store, 1000).await?;
    let row = store
        .get_by_handle("ABC123")
        .await?
        .ok_or_else(|| anyhow::anyhow!("row not upserted — uppercase folder path was dropped"))?;
    assert_eq!(row.text, "hi");
    Ok(())
}
