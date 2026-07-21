//! Cases driven by the metadata-only probe: no-op, wipe-vs-no-wipe on `PbapMeta` changes.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;
use store::{NewContact, PbapMeta};

use super::{body_rsp, fake_store, hex16, list_body, metadata_rsp, sync_contacts, vcard};
use super::{A, B, C, CONNECT_RSP, D};

#[tokio::test]
async fn noop_when_metadata_unchanged() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.set_pbap_meta(&PbapMeta {
        database_id: Some(hex16(A)),
        primary_version: Some(hex16(B)),
        secondary_version: Some(hex16(C)),
    })
    .await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, synced) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(A, B, C)?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(synced?, 0);
    Ok(())
}

#[tokio::test]
async fn refreshes_without_wipe_when_only_counters_change() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![NewContact {
        uid: "uid-old".to_owned(),
        display_name: Some("Old".to_owned()),
        phones: vec!["+15559999".to_owned()],
    }])
    .await?;
    db.set_pbap_meta(&PbapMeta {
        database_id: Some(hex16(A)),
        primary_version: Some(hex16(B)),
        secondary_version: Some(hex16(C)),
    })
    .await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, synced) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(A, D, D)?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["1.vcf"]))?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard(Some("uid-new"), "New", "+15550002"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(synced?, 1);
    assert!(db.get_contact("uid-old").await?.is_some(), "no-wipe path must preserve old contact");
    assert!(db.get_contact("uid-new").await?.is_some());
    Ok(())
}

#[tokio::test]
async fn wipes_when_database_id_changes() -> anyhow::Result<()> {
    let (db, _dir) = fake_store().await?;
    db.upsert_contacts(vec![NewContact {
        uid: "uid-stale".to_owned(),
        display_name: Some("Stale".to_owned()),
        phones: vec!["+15559999".to_owned()],
    }])
    .await?;
    db.set_pbap_meta(&PbapMeta {
        database_id: Some(hex16(A)),
        primary_version: Some(hex16(B)),
        secondary_version: Some(hex16(C)),
    })
    .await?;
    let (client_io, server_io) = tokio::io::duplex(8192);

    let (server_result, synced) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(metadata_rsp(D, D, D)?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["1.vcf"]))?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard(Some("uid-new"), "New", "+15550002"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            sync_contacts(&mut client, &db, PhonebookPath::Pb).await
        },
    );
    server_result?;
    assert_eq!(synced?, 1);
    assert!(db.get_contact("uid-stale").await?.is_none(), "changed database_id must wipe cache");
    assert!(db.get_contact("uid-new").await?.is_some());
    Ok(())
}
