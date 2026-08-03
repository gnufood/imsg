use store::OutgoingStatus;

use super::{event, fake_client, fake_store, handle_mns_event, sample_message, Direction};

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
