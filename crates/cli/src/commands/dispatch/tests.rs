//! `is_opted_in` reads whichever meta key it's given, independently of the others — the fix for
//! contacts silently trusting MAP's `sync_enabled` flag.

use secrecy::SecretBox;
use store::Store;

use super::{canonical_number, is_opted_in};

#[test]
fn canonical_number_normalises_resolvable_input() {
    assert_eq!(canonical_number("+44 (0)1753 866488"), "+441753866488");
}

#[test]
fn canonical_number_falls_back_to_raw_when_unresolvable() {
    assert_eq!(canonical_number("01753 866488"), "01753 866488");
}

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let store = Store::open(dir.path().join("test.db"), key).await?;
    Ok((store, dir))
}

#[tokio::test]
async fn unset_key_is_not_opted_in() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    assert!(!is_opted_in(&db, "sync_enabled").await);
    assert!(!is_opted_in(&db, "contacts_synced").await);
    Ok(())
}

#[tokio::test]
async fn keys_are_tracked_independently() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.set_meta("sync_enabled", "true").await?;

    assert!(is_opted_in(&db, "sync_enabled").await);
    assert!(!is_opted_in(&db, "contacts_synced").await, "contacts must not ride on sync_enabled");

    db.set_meta("contacts_synced", "true").await?;
    assert!(is_opted_in(&db, "contacts_synced").await);
    Ok(())
}
