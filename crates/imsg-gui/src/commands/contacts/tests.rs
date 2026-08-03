//! Exercises the contacts `#[tauri::command]` shims: the three reads directly against a real
//! `Store` (temp-dir `SQLite`, no mocks) via `tauri::test::mock_app()`'s managed `State`, and
//! the sync trigger against a fake broker socket (same approach as `commands::send`'s tests).

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerResponse, RefreshDto, SyncReportDto, MAX_FRAME_LEN};
use secrecy::SecretBox;
use store::{NewContact, PhoneField, Store};
use tauri::Manager;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::*;

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn sample_contact(uid: &str, name: &str, phones: &[&str]) -> NewContact {
    NewContact {
        uid: uid.to_owned(),
        display_name: Some(name.to_owned()),
        phones: phones.iter().map(|p| PhoneField::new(p, None)).collect(),
    }
}

#[tokio::test]
async fn list_contacts_returns_entry_dtos() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![sample_contact("U1", "Ada", &["+15550001"])]).await?;
    let app = tauri::test::mock_app();
    app.manage(db);

    let dtos = list_contacts(app.state::<Store>(), 10, 0).await?;
    assert_eq!(dtos.len(), 1);
    assert_eq!(dtos.first().ok_or_else(|| anyhow::anyhow!("row missing"))?.uid, "U1");
    Ok(())
}

#[tokio::test]
async fn get_contact_returns_none_for_missing() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let app = tauri::test::mock_app();
    app.manage(db);

    assert_eq!(get_contact(app.state::<Store>(), "missing".to_owned()).await?, None);
    Ok(())
}

#[tokio::test]
async fn lookup_contact_returns_dto_for_known_address() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![sample_contact("U1", "Ada", &["+15550001"])]).await?;
    let app = tauri::test::mock_app();
    app.manage(db);

    let dto = lookup_contact(app.state::<Store>(), "+15550001".to_owned())
        .await?
        .ok_or_else(|| anyhow::anyhow!("contact missing"))?;
    assert_eq!(dto.uid, "U1");
    Ok(())
}

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: ipc::BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

#[tokio::test]
async fn sync_contacts_now_returns_report_on_success() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:06:05";
    let listener = bind_for(addr)?;
    let report = SyncReportDto::Refreshed(RefreshDto {
        listed: 2,
        pull_failed: 0,
        no_uid: 0,
        written: 2,
        wiped: false,
    });
    let server = tokio::spawn(serve_one(listener, BrokerResponse::ContactsSynced { report }));

    let got = sync_contacts_now(addr.to_owned()).await?;

    assert_eq!(got, report);
    server.await??;
    Ok(())
}

#[tokio::test]
async fn sync_contacts_now_maps_unreachable_broker_to_command_error() {
    let result = sync_contacts_now("TE:ST:00:00:06:06".to_owned()).await;
    assert!(result.is_err());
}
