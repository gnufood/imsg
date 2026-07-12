//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks). `open_store` itself
//! is not covered here — it requires a live Secret Service D-Bus daemon, same as the CLI's
//! equivalent (`crates/cli/src/commands/mod.rs`), which is untested for the same reason.

use secrecy::SecretBox;
use store::{Direction as StoreDirection, NewMessage, Store};

use super::*;

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn sample_message(handle: &str, address: &str) -> NewMessage {
    NewMessage {
        map_handle: handle.to_owned(),
        timestamp_ms: 1_700_000_000_000,
        folder: "telecom/msg/inbox".to_owned(),
        direction: StoreDirection::Received,
        address: address.to_owned(),
        status: store::STATUS_UNREAD,
        synced_at: 1_700_000_000_500,
        text: "hi".to_owned(),
        outgoing_status: None,
    }
}

#[tokio::test]
async fn get_by_handle_returns_none_for_missing() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    assert_eq!(get_by_handle(&db, "missing").await?, None);
    Ok(())
}

#[tokio::test]
async fn get_by_handle_returns_dto_for_existing() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+15550001")).await?;

    let dto = get_by_handle(&db, "H1").await?.ok_or_else(|| anyhow::anyhow!("row missing"))?;
    assert_eq!(dto.handle, "H1");
    assert_eq!(dto.address, "+15550001");
    Ok(())
}

#[tokio::test]
async fn list_messages_returns_dtos_newest_first() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let mut older = sample_message("H1", "+15550001");
    older.timestamp_ms = 1_700_000_000_000;
    let mut newer = sample_message("H2", "+15550001");
    newer.timestamp_ms = 1_700_000_001_000;
    db.upsert(older).await?;
    db.upsert(newer).await?;

    let dtos = list_messages(&db, None, false, None, None, 10, 0).await?;
    assert_eq!(dtos.len(), 2);
    assert_eq!(dtos.first().ok_or_else(|| anyhow::anyhow!("row missing"))?.handle, "H2");
    assert_eq!(dtos.get(1).ok_or_else(|| anyhow::anyhow!("row missing"))?.handle, "H1");
    Ok(())
}

#[tokio::test]
async fn mark_read_flips_status_to_read() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+15550001")).await?;

    mark_read(&db, "H1").await?;

    let row = db.get_by_handle("H1").await?.ok_or_else(|| anyhow::anyhow!("row missing"))?;
    assert_eq!(row.status, store::STATUS_READ);
    Ok(())
}

#[tokio::test]
async fn threads_aggregates_by_address() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+15550001")).await?;
    db.upsert(sample_message("H2", "+15550001")).await?;
    db.upsert(sample_message("H3", "+15550002")).await?;

    let dtos = threads(&db).await?;
    assert_eq!(dtos.len(), 2);
    let first = dtos
        .iter()
        .find(|t| t.address == "+15550001")
        .ok_or_else(|| anyhow::anyhow!("thread missing"))?;
    assert_eq!(first.total, 2);
    assert_eq!(first.unread, 2);
    Ok(())
}
