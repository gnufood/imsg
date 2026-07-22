//! `disable` reverts both sync domains, not just MAP's `sync_enabled`.

use secrecy::SecretBox;
use store::Store;

use super::disable;

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let store = Store::open(dir.path().join("test.db"), key).await?;
    Ok((store, dir))
}

#[tokio::test]
async fn clears_both_sync_enabled_and_contacts_synced() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.set_meta("sync_enabled", "true").await?;
    db.set_meta("contacts_synced", "true").await?;

    disable(&db).await?;

    assert_eq!(db.get_meta("sync_enabled").await?.as_deref(), Some("false"));
    assert_eq!(db.get_meta("contacts_synced").await?.as_deref(), Some("false"));
    Ok(())
}
