//! Cases driven by the refresh path itself: first sync, UID filtering, and `0.vcf` skipping.

use std::time::Duration;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use store::PbapMeta;

use super::{
    body_rsp, error_rsp, fake_store, hex16, list_body, metadata_rsp, sync_contacts, vcard,
};
use super::{Refresh, SyncReport, A, B, C, CONNECT_RSP};

#[tokio::test]
async fn first_run_fetches_and_persists_meta() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, report) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(A, B, C)?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["1.vcf"]))?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard(Some("uid-1"), "Alice", "+15550001"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(
        report?,
        SyncReport::Refreshed(Refresh {
            listed: 1,
            pull_failed: 0,
            no_uid: 0,
            written: 1,
            wiped: false,
        })
    );

    let contact = db.get_contact("uid-1").await?.ok_or_else(|| anyhow::anyhow!("missing"))?;
    assert_eq!(contact.display_name.as_deref(), Some("Alice"));
    assert_eq!(
        db.pbap_meta().await?,
        PbapMeta {
            database_id: Some(hex16(A)),
            primary_version: Some(hex16(B)),
            secondary_version: Some(hex16(C)),
        }
    );
    Ok(())
}

#[tokio::test]
async fn skips_entries_without_uid() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, report) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(A, B, C)?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["1.vcf", "2.vcf"]))?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard(None, "NoUid", "+15550001"))?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard(Some("uid-2"), "HasUid", "+15550002"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(
        report?,
        SyncReport::Refreshed(Refresh {
            listed: 2,
            pull_failed: 0,
            no_uid: 1,
            written: 1,
            wiped: false,
        })
    );
    assert!(db.get_contact("uid-2").await?.is_some());
    Ok(())
}

/// A per-entry pull failure is counted, not swallowed: the sync still succeeds and writes the
/// entries that did arrive, but the report says one was lost — the distinction `count` alone
/// could never express.
#[tokio::test]
async fn counts_failed_pulls_without_aborting() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, report) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(A, B, C)?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["1.vcf", "2.vcf"]))?).await?;
            let _ = srv.next().await;
            srv.send(error_rsp()?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard(Some("uid-2"), "Reached", "+15550002"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(
        report?,
        SyncReport::Refreshed(Refresh {
            listed: 2,
            pull_failed: 1,
            no_uid: 0,
            written: 1,
            wiped: false,
        })
    );
    assert!(db.get_contact("uid-2").await?.is_some());
    Ok(())
}

#[tokio::test]
async fn skips_zero_vcf_entry() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, report) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(A, B, C)?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["0.vcf"]))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            tokio::time::timeout(
                Duration::from_secs(2),
                sync_contacts(&mut client, &db, PhonebookPath::Pb),
            )
            .await
            .map_err(|_| anyhow::anyhow!("sync_contacts hung — 0.vcf was not skipped"))?
        },
    );
    server_result?;
    // The owner card counts as listed but is never pulled — it has no counter of its own, so
    // `listed` deliberately doesn't reconcile with the other three here.
    assert_eq!(
        report?,
        SyncReport::Refreshed(Refresh {
            listed: 1,
            pull_failed: 0,
            no_uid: 0,
            written: 0,
            wiped: false,
        })
    );
    Ok(())
}
