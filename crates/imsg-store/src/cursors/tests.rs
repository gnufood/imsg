//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks).

use secrecy::SecretBox;

use crate::{FolderSyncStatus, Store};

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

#[tokio::test]
async fn latest_sync_at_is_none_before_any_folder_syncs() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    assert_eq!(db.latest_sync_at().await?, None);
    Ok(())
}

#[tokio::test]
async fn latest_sync_at_is_max_across_folders() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    db.set_cursor("inbox", 1_000, 0, FolderSyncStatus::Complete).await?;
    db.set_cursor("sent", 3_000, 0, FolderSyncStatus::Complete).await?;
    db.set_cursor("deleted", 2_000, 0, FolderSyncStatus::Complete).await?;

    assert_eq!(db.latest_sync_at().await?, Some(3_000));
    Ok(())
}

#[tokio::test]
async fn latest_sync_at_reflects_a_single_folder_rescan() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.set_cursor("inbox", 1_000, 0, FolderSyncStatus::Complete).await?;

    db.set_cursor("inbox", 5_000, 0, FolderSyncStatus::Complete).await?;

    assert_eq!(db.latest_sync_at().await?, Some(5_000));
    Ok(())
}
