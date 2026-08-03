//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks).

use formats::phone::PhoneField;
use secrecy::SecretBox;

use crate::{Direction, NewContact, NewMessage, Store};

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
        direction: Direction::Received,
        address: PhoneField::new(address, None),
        status: crate::STATUS_UNREAD,
        synced_at: 1_700_000_000_500,
        text: "hi".to_owned(),
        outgoing_status: None,
    }
}

#[tokio::test]
async fn threads_carries_cached_contact_name_when_present() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+15550001")).await?;
    db.upsert_contacts(vec![NewContact {
        uid: "uid-alice".to_owned(),
        display_name: Some("Alice".to_owned()),
        phones: vec![PhoneField::new("+15550001", None)],
    }])
    .await?;

    let rows = db.threads().await?;

    let row = rows
        .iter()
        .find(|t| t.address == "+15550001")
        .ok_or_else(|| anyhow::anyhow!("thread missing"))?;
    assert_eq!(row.contact_name.as_deref(), Some("Alice"));
    Ok(())
}

#[tokio::test]
async fn list_from_matches_across_formatting_via_canonical() -> anyhow::Result<()> {
    // stored raw ("+44(0)1753866488") and the query ("+441753866488") differ in formatting but
    // normalise to the same E.164, so the filter must match on canonical
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+44(0)1753866488")).await?;

    let got = db.list_messages(None, false, Some("+441753866488"), None, 10, 0).await?;

    assert_eq!(got.len(), 1);
    Ok(())
}

#[tokio::test]
async fn threads_joins_contact_across_formatting_via_canonical() -> anyhow::Result<()> {
    // message address and contact phone differ in formatting (trunk prefix) but share an E.164,
    // so the contact_phones join must match on canonical
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+44(0)1753866488")).await?;
    db.upsert_contacts(vec![NewContact {
        uid: "uid-ada".to_owned(),
        display_name: Some("Ada".to_owned()),
        phones: vec![PhoneField::new("+441753866488", None)],
    }])
    .await?;

    let rows = db.threads().await?;

    let row = rows.first().ok_or_else(|| anyhow::anyhow!("thread missing"))?;
    assert_eq!(row.contact_name.as_deref(), Some("Ada"));
    Ok(())
}

#[tokio::test]
async fn threads_carries_none_contact_name_when_no_cached_contact() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert(sample_message("H1", "+15550001")).await?;

    let rows = db.threads().await?;

    let row = rows
        .iter()
        .find(|t| t.address == "+15550001")
        .ok_or_else(|| anyhow::anyhow!("thread missing"))?;
    assert_eq!(row.contact_name, None);
    Ok(())
}
