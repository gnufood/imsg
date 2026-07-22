//! Unit tests for the `store` row -> GUI DTO conversions, plus one end-to-end check against a
//! real `Store` (temp-dir `SQLite`, no mocks).

use secrecy::SecretBox;
use store::{
    Direction as StoreDirection, NewMessage, OutgoingStatus as StoreOutgoingStatus, Store,
};

use super::*;

fn sample_row(status: i32, outgoing_status: Option<StoreOutgoingStatus>) -> store::MessageRow {
    store::MessageRow {
        rowid: 7,
        map_handle: "H1".into(),
        timestamp_ms: 1_700_000_000_000,
        folder: "telecom/msg/inbox".into(),
        direction: StoreDirection::Received,
        address: "+15550001".into(),
        status,
        synced_at: 1_700_000_000_500,
        text: "hi".into(),
        outgoing_status,
    }
}

#[test]
fn message_dto_marks_unread_status_as_unread() {
    let dto = MessageDto::from(&sample_row(store::STATUS_UNREAD, None));
    assert!(!dto.read);
}

#[test]
fn message_dto_marks_read_status_as_read() {
    let dto = MessageDto::from(&sample_row(store::STATUS_READ, None));
    assert!(dto.read);
}

#[test]
fn message_dto_preserves_core_fields() {
    let row = sample_row(store::STATUS_READ, None);
    let dto = MessageDto::from(&row);
    assert_eq!(dto.handle, "H1");
    assert_eq!(dto.timestamp_ms, 1_700_000_000_000);
    assert_eq!(dto.folder, "telecom/msg/inbox");
    assert_eq!(dto.direction, Direction::Received);
    assert_eq!(dto.address, "+15550001");
    assert_eq!(dto.text, "hi");
}

#[test]
fn message_dto_maps_none_outgoing_status() {
    let dto = MessageDto::from(&sample_row(store::STATUS_READ, None));
    assert_eq!(dto.outgoing_status, None);
}

/// Every `store::OutgoingStatus` variant must map to its `dto::OutgoingStatus` counterpart —
/// exhaustive in the `From` impl, so a new store variant fails to compile here.
#[test]
fn outgoing_status_maps_every_variant() {
    let cases = [
        (StoreOutgoingStatus::Queued, OutgoingStatus::Queued),
        (StoreOutgoingStatus::Sending, OutgoingStatus::Sending),
        (StoreOutgoingStatus::SentUnconfirmed, OutgoingStatus::SentUnconfirmed),
        (StoreOutgoingStatus::SentConfirmed, OutgoingStatus::SentConfirmed),
        (StoreOutgoingStatus::FailedRetryable, OutgoingStatus::FailedRetryable),
        (StoreOutgoingStatus::FailedPermanent, OutgoingStatus::FailedPermanent),
        (StoreOutgoingStatus::Unknown, OutgoingStatus::Unknown),
    ];
    for (store_status, expected) in cases {
        let dto = MessageDto::from(&sample_row(store::STATUS_READ, Some(store_status)));
        assert_eq!(dto.outgoing_status, Some(expected));
    }
}

#[test]
fn direction_maps_both_variants() {
    assert_eq!(Direction::from(StoreDirection::Received), Direction::Received);
    assert_eq!(Direction::from(StoreDirection::Sent), Direction::Sent);
}

#[test]
fn thread_dto_preserves_fields_and_maps_outgoing_status() {
    let row = store::ThreadRow {
        address: "+15550001".into(),
        latest_ms: 1_700_000_000_000,
        total: 4,
        unread: 2,
        latest_outgoing_status: Some(StoreOutgoingStatus::SentConfirmed),
        contact_name: None,
    };
    let dto = ThreadDto::from(&row);
    assert_eq!(dto.address, "+15550001");
    assert_eq!(dto.latest_ms, 1_700_000_000_000);
    assert_eq!(dto.total, 4);
    assert_eq!(dto.unread, 2);
    assert_eq!(dto.latest_outgoing_status, Some(OutgoingStatus::SentConfirmed));
    assert_eq!(dto.contact_name, None);
}

#[test]
fn thread_dto_carries_cached_contact_name() {
    let row = store::ThreadRow {
        address: "+15550001".into(),
        latest_ms: 1_700_000_000_000,
        total: 1,
        unread: 0,
        latest_outgoing_status: None,
        contact_name: Some("Jane Doe".into()),
    };
    let dto = ThreadDto::from(&row);
    assert_eq!(dto.contact_name.as_deref(), Some("Jane Doe"));
}

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// End-to-end against a real store: upsert a sent message, read it back via `list_messages`,
/// convert, and check the DTO reflects what the CLI's own local-read path would show.
#[tokio::test]
async fn message_dto_reflects_real_store_row() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    store
        .upsert(NewMessage {
            map_handle: "H1".to_owned(),
            timestamp_ms: 1_700_000_000_000,
            folder: "telecom/msg/sent".to_owned(),
            direction: StoreDirection::Sent,
            address: "+15550001".to_owned(),
            status: store::STATUS_READ,
            synced_at: 1_700_000_000_500,
            text: "on my way".to_owned(),
            outgoing_status: Some(StoreOutgoingStatus::SentConfirmed),
        })
        .await?;

    let rows = store.list_messages(None, false, None, None, 10, 0).await?;
    let row = rows.first().ok_or_else(|| anyhow::anyhow!("row unexpectedly missing"))?;
    let dto = MessageDto::from(row);

    assert_eq!(dto.handle, "H1");
    assert_eq!(dto.direction, Direction::Sent);
    assert!(dto.read);
    assert_eq!(dto.outgoing_status, Some(OutgoingStatus::SentConfirmed));
    Ok(())
}

#[test]
fn paired_device_dto_formats_address_and_keeps_name() -> anyhow::Result<()> {
    let device = transport::discover::PairedDevice {
        address: "AA:BB:CC:DD:EE:FF".parse()?,
        name: Some("Test Phone".to_owned()),
    };
    let dto = PairedDeviceDto::from(&device);
    assert_eq!(dto.address, "AA:BB:CC:DD:EE:FF");
    assert_eq!(dto.name.as_deref(), Some("Test Phone"));
    Ok(())
}

#[test]
fn paired_device_dto_keeps_missing_name_absent() -> anyhow::Result<()> {
    let device =
        transport::discover::PairedDevice { address: "00:00:00:00:00:00".parse()?, name: None };
    assert_eq!(PairedDeviceDto::from(&device).name, None);
    Ok(())
}

/// Every `config::SecurityLevel` variant must round-trip through `SecurityLevelDto` both ways
/// — exhaustive in both `From` impls, so a new config variant fails to compile here.
#[test]
fn security_level_dto_maps_every_variant_both_ways() {
    let cases = [
        (config::SecurityLevel::Sdp, SecurityLevelDto::Sdp),
        (config::SecurityLevel::Low, SecurityLevelDto::Low),
        (config::SecurityLevel::Medium, SecurityLevelDto::Medium),
        (config::SecurityLevel::High, SecurityLevelDto::High),
    ];
    for (cfg_level, dto_level) in cases {
        assert_eq!(SecurityLevelDto::from(cfg_level), dto_level);
        assert_eq!(config::SecurityLevel::from(dto_level), cfg_level);
    }
}

#[test]
fn channels_dto_preserves_found_and_missing_channels() {
    let channels = transport::discover::Channels { map: Some(2), pbap: None };
    let dto = ChannelsDto::from(channels);
    assert_eq!(dto.map, Some(2));
    assert_eq!(dto.pbap, None);
}
