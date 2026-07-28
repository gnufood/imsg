//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks).

use formats::phone::PhoneField;
use secrecy::SecretBox;

use crate::{ContactRow, NewContact, PbapMeta, Store};

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn alice() -> NewContact {
    NewContact {
        uid: "uid-alice".to_owned(),
        display_name: Some("Alice".to_owned()),
        phones: vec![PhoneField::new("+15550001", None), PhoneField::new("+15550002", None)],
    }
}

#[tokio::test]
async fn upsert_contacts_writes_contact_and_phones() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    db.upsert_contacts(vec![alice()]).await?;

    let got = db.get_contact("uid-alice").await?.ok_or_else(|| anyhow::anyhow!("missing"))?;
    assert_eq!(
        got,
        ContactRow {
            uid: "uid-alice".to_owned(),
            display_name: Some("Alice".to_owned()),
            phones: vec![PhoneField::new("+15550001", None), PhoneField::new("+15550002", None)],
        }
    );
    Ok(())
}

#[tokio::test]
async fn upsert_contacts_replaces_phone_set_on_second_call() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![alice()]).await?;

    db.upsert_contacts(vec![NewContact {
        uid: "uid-alice".to_owned(),
        display_name: Some("Alice Smith".to_owned()),
        phones: vec![PhoneField::new("+15559999", None)],
    }])
    .await?;

    let got = db.get_contact("uid-alice").await?.ok_or_else(|| anyhow::anyhow!("missing"))?;
    assert_eq!(got.display_name.as_deref(), Some("Alice Smith"));
    assert_eq!(got.phones, vec![PhoneField::new("+15559999", None)]);
    Ok(())
}

#[tokio::test]
async fn upsert_persists_e164_column_and_round_trips() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let phone = PhoneField::new("+44 (0)1753 866488", None);
    assert_eq!(phone.e164(), Some("+441753866488"));

    db.upsert_contacts(vec![NewContact {
        uid: "uid-ada".to_owned(),
        display_name: Some("Ada".to_owned()),
        phones: vec![phone],
    }])
    .await?;

    let got = db.get_contact("uid-ada").await?.ok_or_else(|| anyhow::anyhow!("missing"))?;
    let phone = got.phones.first().ok_or_else(|| anyhow::anyhow!("no phone"))?;
    assert_eq!(phone.raw(), "+44 (0)1753 866488");
    assert_eq!(phone.e164(), Some("+441753866488"));
    Ok(())
}

#[tokio::test]
async fn get_contact_returns_none_when_absent() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    assert!(db.get_contact("no-such-uid").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn lookup_contact_finds_owner_by_phone_number() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![alice()]).await?;

    let got = db.lookup_contact("+15550002").await?.ok_or_else(|| anyhow::anyhow!("missing"))?;

    assert_eq!(got.uid, "uid-alice");
    Ok(())
}

#[tokio::test]
async fn lookup_contact_returns_none_when_no_match() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    assert!(db.lookup_contact("+19998887777").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn list_contacts_orders_by_display_name_then_uid() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![
        NewContact {
            uid: "uid-b".to_owned(),
            display_name: Some("Bob".to_owned()),
            phones: vec![],
        },
        NewContact {
            uid: "uid-a".to_owned(),
            display_name: Some("alice".to_owned()),
            phones: vec![],
        },
    ])
    .await?;

    let entries = db.list_contacts(10, 0).await?;

    let names: Vec<Option<String>> = entries.into_iter().map(|e| e.display_name).collect();
    assert_eq!(names, vec![Some("alice".to_owned()), Some("Bob".to_owned())]);
    Ok(())
}

#[tokio::test]
async fn list_contacts_paginates_with_limit_and_offset() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![
        NewContact { uid: "uid-a".to_owned(), display_name: Some("A".to_owned()), phones: vec![] },
        NewContact { uid: "uid-b".to_owned(), display_name: Some("B".to_owned()), phones: vec![] },
        NewContact { uid: "uid-c".to_owned(), display_name: Some("C".to_owned()), phones: vec![] },
    ])
    .await?;

    let page = db.list_contacts(1, 1).await?;

    assert_eq!(page.len(), 1);
    assert_eq!(page.first().map(|e| e.uid.as_str()), Some("uid-b"));
    Ok(())
}

#[tokio::test]
async fn all_contacts_returns_full_rows_with_phones() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![alice()]).await?;

    let all = db.all_contacts(10, 0).await?;

    assert_eq!(all.len(), 1);
    let first = all.first().ok_or_else(|| anyhow::anyhow!("missing"))?;
    assert_eq!(
        first.phones,
        vec![PhoneField::new("+15550001", None), PhoneField::new("+15550002", None)]
    );
    Ok(())
}

#[tokio::test]
async fn clear_contacts_wipes_both_tables() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![alice()]).await?;

    db.clear_contacts().await?;

    assert!(db.get_contact("uid-alice").await?.is_none());
    assert!(db.lookup_contact("+15550001").await?.is_none());
    Ok(())
}

#[tokio::test]
async fn pbap_meta_round_trips() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let meta = PbapMeta {
        database_id: Some("aa".to_owned()),
        primary_version: Some("bb".to_owned()),
        secondary_version: Some("cc".to_owned()),
    };

    db.set_pbap_meta(&meta).await?;

    assert_eq!(db.pbap_meta().await?, meta);
    Ok(())
}

#[tokio::test]
async fn pbap_meta_defaults_to_none_when_never_set() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    assert_eq!(db.pbap_meta().await?, PbapMeta::default());
    Ok(())
}

#[tokio::test]
async fn contacts_synced_at_is_none_before_any_sync() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    assert_eq!(db.contacts_synced_at().await?, None);
    Ok(())
}

#[tokio::test]
async fn contacts_synced_at_returns_last_set_value() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    db.set_contacts_synced_at(1_000).await?;
    db.set_contacts_synced_at(2_000).await?;

    assert_eq!(db.contacts_synced_at().await?, Some(2_000));
    Ok(())
}
