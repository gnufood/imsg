//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks).

use secrecy::SecretBox;

use crate::Store;

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

#[tokio::test]
async fn upsert_contact_inserts_new_row() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    db.upsert_contact("+15550001", Some("Alice")).await?;

    assert_eq!(db.contact_name("+15550001").await?.as_deref(), Some("Alice"));
    Ok(())
}

#[tokio::test]
async fn upsert_contact_overwrites_existing_display_name() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    db.upsert_contact("+15550001", Some("Alice")).await?;
    db.upsert_contact("+15550001", Some("Alice Smith")).await?;

    assert_eq!(db.contact_name("+15550001").await?.as_deref(), Some("Alice Smith"));
    Ok(())
}

#[tokio::test]
async fn upsert_contact_accepts_absent_display_name() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    db.upsert_contact("+15550001", None).await?;

    assert_eq!(db.contact_name("+15550001").await?, None);
    Ok(())
}

#[tokio::test]
async fn contact_name_returns_none_when_no_contact_cached() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    assert_eq!(db.contact_name("+15559999999").await?, None);
    Ok(())
}

#[tokio::test]
async fn upsert_contacts_writes_every_pair_in_one_transaction() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    let written = db
        .upsert_contacts(vec![
            ("+15550001".to_owned(), Some("Alice".to_owned())),
            ("+15550002".to_owned(), None),
        ])
        .await?;

    assert_eq!(written, 2);
    assert_eq!(db.contact_name("+15550001").await?.as_deref(), Some("Alice"));
    assert_eq!(db.contact_name("+15550002").await?, None);
    Ok(())
}

#[tokio::test]
async fn upsert_contacts_overwrites_existing_display_name() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contact("+15550001", Some("Old Name")).await?;

    db.upsert_contacts(vec![("+15550001".to_owned(), Some("New Name".to_owned()))]).await?;

    assert_eq!(db.contact_name("+15550001").await?.as_deref(), Some("New Name"));
    Ok(())
}
