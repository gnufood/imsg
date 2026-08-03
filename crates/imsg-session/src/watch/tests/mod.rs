//! Shared fake-`MapClient` harness for [`super::handle_mns_event`] tests, driven over a
//! `tokio::io::duplex` pair against a real (temp-dir) `Store`.

mod folder_and_status;
mod new_message;
mod outgoing;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use map_core::client::MapClient;
use map_core::mns_event::parse_event_report;
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use secrecy::SecretBox;
use store::{NewMessage, PhoneField, Store, STATUS_READ};
use tokio::io::{duplex, DuplexStream};

use super::*;

const CONNECT_RSP: &[u8] = include_bytes!("../../../../imsg-obex/tests/fixtures/connect_rsp.bin");
const OK_RSP: &[u8] = &[0xA0, 0x00, 0x03];

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn sample_message(direction: Direction, outgoing_status: Option<OutgoingStatus>) -> NewMessage {
    let folder =
        if direction == Direction::Sent { "telecom/msg/sent" } else { "telecom/msg/inbox" };
    NewMessage {
        map_handle: "H1".to_owned(),
        timestamp_ms: 0,
        folder: folder.to_owned(),
        direction,
        address: PhoneField::new("+1", None),
        status: STATUS_READ,
        synced_at: 0,
        text: "hi".to_owned(),
        outgoing_status,
    }
}

// fake server only answers CONNECT — sufficient for every event type except NewMessage, the
// only branch that issues further MAP requests
async fn fake_client() -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
    });
    Ok(MapClient::connect(client_io).await?)
}

// fake server also answers the 3-segment set_folder SETPATH sequence and a GetMessage GET,
// replying with wire as the bMessage body
async fn fake_client_with_message(wire: String) -> anyhow::Result<MapClient<DuplexStream>> {
    let (client_io, server_io) = duplex(4096);
    tokio::spawn(async move {
        let mut srv = obex_core::wrap(server_io);
        let _ = srv.next().await;
        let _ = srv.send(Bytes::from_static(CONNECT_RSP)).await;
        for _ in 0..3 {
            let _ = srv.next().await;
            let _ = srv.send(Bytes::from_static(OK_RSP)).await;
        }
        let _ = srv.next().await;
        if let Ok(pkt) = (Packet {
            opcode: OpCode::Ok,
            extra: PacketExtra::None,
            headers: vec![Header::EndOfBody(Bytes::from(wire.into_bytes()))],
        })
        .encode()
        {
            let _ = srv.send(pkt).await;
        }
    });
    Ok(MapClient::connect(client_io).await?)
}

fn event(xml: &str) -> anyhow::Result<MnsEvent> {
    Ok(parse_event_report(xml.as_bytes())?)
}
