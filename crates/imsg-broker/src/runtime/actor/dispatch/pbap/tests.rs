//! Unit tests for [`super::do_sync_contacts`].

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use ipc::{CardEntryDto, ContactDto, PhoneDto, Reason};
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use pbap_core::client::PbapClient;
use secrecy::SecretBox;
use store::Store;
use tokio::io::{duplex, DuplexStream};

use super::*;

// OBEX CONNECT OK response with a ConnectionId header (same fixture bytes as imsg-pbap's
// `tests/fixtures/pbap_connect_rsp.bin`).
const PBAP_CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// Encodes a single-response OBEX OK packet carrying `body` as `EndOfBody`.
fn ok_body_packet(body: &[u8]) -> anyhow::Result<Bytes> {
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()?)
}

/// A connected PBAP client whose fake server answers CONNECT, then `sync_contacts`'s
/// metadata-only probe (all-`None`, forcing a refresh), then one `list` (one `1.vcf` entry) and
/// one `pull` returning `vcard`.
async fn fake_pbap_client(vcard: &'static [u8]) -> anyhow::Result<PbapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(PBAP_CONNECT_RSP)).await;
        let _ = srv.next().await;
        if let Ok(pkt) = ok_body_packet(&[]) {
            let _ = srv.send(pkt).await;
        }
        let _ = srv.next().await;
        let listing = b"<vCard-listing><card handle=\"1.vcf\" name=\"x\"/></vCard-listing>";
        if let Ok(pkt) = ok_body_packet(listing) {
            let _ = srv.send(pkt).await;
        }
        let _ = srv.next().await;
        if let Ok(pkt) = ok_body_packet(vcard) {
            let _ = srv.send(pkt).await;
        }
    });
    Ok(PbapClient::connect(client_io).await?)
}

/// In-memory `Store` (temp-dir `SQLite`) plus the dir guard.
async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

/// A full sync writes every phone number on the pulled contact into the store cache and reports
/// the row count.
#[tokio::test]
async fn do_sync_contacts_upserts_and_reports_count() -> anyhow::Result<()> {
    let (store, _dir) = fake_store().await?;
    let body = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nUID:uid-alice\r\n\
TEL:+15551110000\r\nEND:VCARD\r\n";
    let mut pbap = fake_pbap_client(body).await?;

    let resp = do_sync_contacts(&mut pbap, &store).await?;

    assert!(matches!(resp, BrokerResponse::ContactsSynced { count: 1 }));
    let contact =
        store.lookup_contact("+15551110000").await?.ok_or_else(|| anyhow::anyhow!("missing"))?;
    assert_eq!(contact.display_name.as_deref(), Some("Alice"));
    Ok(())
}

/// A connected PBAP client whose fake server answers CONNECT, then one further GET with
/// `body` as its `EndOfBody`.
async fn fake_pbap_client_one_shot(
    body: &'static [u8],
) -> anyhow::Result<PbapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(PBAP_CONNECT_RSP)).await;
        let _ = srv.next().await;
        if let Ok(pkt) = ok_body_packet(body) {
            let _ = srv.send(pkt).await;
        }
    });
    Ok(PbapClient::connect(client_io).await?)
}

/// An unrecognized `path` name fails without ever touching the PBAP transport — the fake
/// server here only answers CONNECT, proving no further request went out.
#[tokio::test]
async fn unknown_path_fails_without_touching_device() -> anyhow::Result<()> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(PBAP_CONNECT_RSP)).await;
    });
    let mut pbap = PbapClient::connect(client_io).await?;

    let resp = do_contacts_list(&mut pbap, Some("bogus".into()), None, 0).await?;

    assert!(matches!(resp, BrokerResponse::Failed(Reason::OperationFailed(_))));
    Ok(())
}

/// Lists a phonebook and maps each listing entry to a [`CardEntryDto`].
#[tokio::test]
async fn do_contacts_list_returns_entries() -> anyhow::Result<()> {
    let listing = b"<vCard-listing><card handle=\"1.vcf\" name=\"Alice\"/></vCard-listing>";
    let mut pbap = fake_pbap_client_one_shot(listing).await?;

    let resp = do_contacts_list(&mut pbap, None, None, 0).await?;

    let BrokerResponse::ContactEntries(entries) = resp else {
        anyhow::bail!("unexpected response: {resp:?}");
    };
    assert_eq!(entries, vec![CardEntryDto { handle: "1.vcf".into(), name: Some("Alice".into()) }]);
    Ok(())
}

/// Fetches one contact vCard by handle and maps it to a [`ContactDto`].
#[tokio::test]
async fn do_contacts_get_returns_contact() -> anyhow::Result<()> {
    let vcard = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nUID:uid-alice\r\n\
TEL:+15551110000\r\nEND:VCARD\r\n";
    let mut pbap = fake_pbap_client_one_shot(vcard).await?;

    let resp = do_contacts_get(&mut pbap, None, "1.vcf".into()).await?;

    let BrokerResponse::Contact(contact) = resp else {
        anyhow::bail!("unexpected response: {resp:?}");
    };
    assert_eq!(
        contact,
        ContactDto {
            display_name: Some("Alice".into()),
            uid: Some("uid-alice".into()),
            phones: vec![PhoneDto { raw: "+15551110000".to_owned(), e164: None }],
        }
    );
    Ok(())
}

/// Pulls every contact in a phonebook and maps each to a [`ContactDto`].
#[tokio::test]
async fn do_contacts_pull_all_returns_contacts() -> anyhow::Result<()> {
    let vcard = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nUID:uid-alice\r\n\
TEL:+15551110000\r\nEND:VCARD\r\n";
    let mut pbap = fake_pbap_client_one_shot(vcard).await?;

    let resp = do_contacts_pull_all(&mut pbap, None, None, 0).await?;

    let BrokerResponse::Contacts(contacts) = resp else {
        anyhow::bail!("unexpected response: {resp:?}");
    };
    assert_eq!(
        contacts,
        vec![ContactDto {
            display_name: Some("Alice".into()),
            uid: Some("uid-alice".into()),
            phones: vec![PhoneDto { raw: "+15551110000".to_owned(), e164: None }],
        }]
    );
    Ok(())
}

/// A device-side search hit is resolved into the matching contact's vCard.
#[tokio::test]
async fn do_contacts_lookup_finds_match() -> anyhow::Result<()> {
    let listing = b"<vCard-listing><card handle=\"1.vcf\" name=\"Alice\"/></vCard-listing>";
    let vcard = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Alice\r\nUID:uid-alice\r\n\
TEL:+15551110000\r\nEND:VCARD\r\n";
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(PBAP_CONNECT_RSP)).await;
        let _ = srv.next().await;
        if let Ok(pkt) = ok_body_packet(listing) {
            let _ = srv.send(pkt).await;
        }
        let _ = srv.next().await;
        if let Ok(pkt) = ok_body_packet(vcard) {
            let _ = srv.send(pkt).await;
        }
    });
    let mut pbap = PbapClient::connect(client_io).await?;

    let resp = do_contacts_lookup(&mut pbap, None, "+15551110000".into()).await?;

    let BrokerResponse::ContactLookup(Some(contact)) = resp else {
        anyhow::bail!("unexpected response: {resp:?}");
    };
    assert_eq!(contact.display_name.as_deref(), Some("Alice"));
    Ok(())
}

/// No device-side search hit resolves to `None`, with no follow-up pull.
#[tokio::test]
async fn do_contacts_lookup_no_match_returns_none() -> anyhow::Result<()> {
    let listing = b"<vCard-listing></vCard-listing>";
    let mut pbap = fake_pbap_client_one_shot(listing).await?;

    let resp = do_contacts_lookup(&mut pbap, None, "+15559998888".into()).await?;

    assert!(matches!(resp, BrokerResponse::ContactLookup(None)));
    Ok(())
}
