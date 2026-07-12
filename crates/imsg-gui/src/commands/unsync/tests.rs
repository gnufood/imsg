//! Exercises `unsync_disable` against a real `Store` and a real `tauri::test::mock_app()`.

use secrecy::SecretBox;
use store::Store;
use tauri::Manager;

use super::*;

#[tokio::test]
async fn unsync_disable_clears_sync_enabled() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let db = Store::open(dir.path().join("test.db"), key).await?;
    db.set_meta("sync_enabled", "true").await?;
    let app = tauri::test::mock_app();
    app.manage(db);

    unsync_disable(app.state::<Store>()).await?;

    assert_eq!(app.state::<Store>().get_meta("sync_enabled").await?, Some("false".to_owned()));
    Ok(())
}
