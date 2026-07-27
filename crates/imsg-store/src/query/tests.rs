//! End-to-end checks against a real `Store` (temp-dir `SQLite`, no mocks).

use secrecy::SecretBox;

use crate::row::{Direction, OutboxStatus};
use crate::{NewMessage, PhoneField, Store};

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn sample_outgoing() -> NewMessage {
    NewMessage {
        map_handle: String::new(),
        timestamp_ms: 1_700_000_000_000,
        folder: "telecom/msg/sent".to_owned(),
        direction: Direction::Sent,
        address: PhoneField::new("+15550001", None),
        status: crate::STATUS_READ,
        synced_at: 1_700_000_000_500,
        text: "hi".to_owned(),
        outgoing_status: None,
    }
}

/// Regression test for a hard `FOREIGN KEY constraint failed` on `delete_by_handle`: a message
/// that went through the outbox (`enqueue_send` links `outbox.local_message_id` to it) must
/// remain deletable even though an outbox row still references its rowid.
#[tokio::test]
async fn delete_by_handle_succeeds_for_a_message_linked_from_outbox() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;

    let (_msg_rowid, outbox_id) = db
        .enqueue_send(sample_outgoing(), "send_sms", "+15550001\x1Fhi", 1_700_000_000_000)
        .await?;
    let placeholder = format!("local:{outbox_id}");
    db.resolve(outbox_id, OutboxStatus::Sent, 1_700_000_000_600, None).await?;

    db.delete_by_handle(&placeholder).await?;

    assert!(db.get_by_handle(&placeholder).await?.is_none());
    Ok(())
}
