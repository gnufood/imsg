//! Drives a fake PBAP server over a `tokio::io::duplex` pair through the shared live wrappers.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use pbap_core::client::PbapClient;
use pbap_core::phonebook::PhonebookPath;

use super::{get, list, lookup, pull_all};

const CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

fn body_rsp(body: &[u8]) -> anyhow::Result<Bytes> {
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()?)
}

fn list_body(handles: &[&str]) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut body = String::from("<vCard-listing>");
    for h in handles {
        let _ = write!(body, "<card handle=\"{h}\" name=\"x\"/>");
    }
    body.push_str("</vCard-listing>");
    body.into_bytes()
}

fn vcard(name: &str, tel: &str) -> Vec<u8> {
    format!("BEGIN:VCARD\r\nVERSION:3.0\r\nFN:{name}\r\nTEL:{tel}\r\nEND:VCARD\r\n").into_bytes()
}

#[tokio::test]
async fn list_returns_card_entries() -> anyhow::Result<()> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, entries) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["1.vcf"]))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            list(&mut client, PhonebookPath::Pb, None, 0).await
        },
    );
    server_result?;
    let entries = entries?;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries.first().map(pbap_core::CardEntry::handle), Some("1.vcf"));
    Ok(())
}

#[tokio::test]
async fn get_returns_contact() -> anyhow::Result<()> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, contact) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard("Bob", "+15551234567"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            get(&mut client, PhonebookPath::Pb, "5.vcf").await
        },
    );
    server_result?;
    assert_eq!(contact?.display_name.as_deref(), Some("Bob"));
    Ok(())
}

#[tokio::test]
async fn pull_all_returns_contacts() -> anyhow::Result<()> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, contacts) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard("Alice", "+15550001"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            pull_all(&mut client, PhonebookPath::Pb, None, 0).await
        },
    );
    server_result?;
    assert_eq!(contacts?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn lookup_pulls_first_non_owner_match() -> anyhow::Result<()> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, found) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&["0.vcf", "3.vcf"]))?).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&vcard("Carol", "+15559990000"))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            lookup(&mut client, PhonebookPath::Pb, "+15559990000").await
        },
    );
    server_result?;
    let contact = found?.ok_or_else(|| anyhow::anyhow!("expected a match"))?;
    assert_eq!(contact.display_name.as_deref(), Some("Carol"));
    Ok(())
}

#[tokio::test]
async fn lookup_returns_none_when_search_is_empty() -> anyhow::Result<()> {
    let (client_io, server_io) = tokio::io::duplex(4096);

    let (server_result, found) = futures::join!(
        async {
            let mut srv = obex_core::wrap(server_io);
            let _ = srv.next().await;
            srv.send(Bytes::from_static(CONNECT_RSP)).await?;
            let _ = srv.next().await;
            srv.send(body_rsp(&list_body(&[]))?).await?;
            Ok::<(), anyhow::Error>(())
        },
        async {
            let mut client = PbapClient::connect(client_io).await?;
            lookup(&mut client, PhonebookPath::Pb, "+19998887777").await
        },
    );
    server_result?;
    assert_eq!(found?, None);
    Ok(())
}
