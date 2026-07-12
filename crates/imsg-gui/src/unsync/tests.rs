//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks).

use secrecy::SecretBox;
use store::Store;

use super::*;

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir, PathBuf)> {
    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("test.db");
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(db_path.clone(), key).await?;
    Ok((s, dir, db_path))
}

#[tokio::test]
async fn disable_clears_sync_enabled() -> anyhow::Result<()> {
    let (store, _dir, _path) = fake_store().await?;
    store.set_meta("sync_enabled", "true").await?;

    disable(&store).await?;

    assert_eq!(store.get_meta("sync_enabled").await?, Some("false".to_owned()));
    Ok(())
}

#[tokio::test]
async fn purge_removes_db_file() -> anyhow::Result<()> {
    let (store, _dir, db_path) = fake_store().await?;
    drop(store);

    purge(db_path.clone())?;

    assert!(!db_path.exists());
    Ok(())
}

#[test]
fn purge_errors_when_db_file_missing() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let missing = dir.path().join("nonexistent.db");
    assert!(purge(missing).is_err());
    Ok(())
}
