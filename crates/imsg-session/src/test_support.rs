//! Shared OBEX client-side helpers for MNS tests — drives the device half of a duplex pair.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use map_core::mns_event::MnsError;
use map_core::ObexError;
use obex_core::{
    client::ObexClient,
    headers::Header,
    packet::{OpCode, Packet, PacketExtra},
};
use tokio::io::DuplexStream;

/// MNS Target UUID the server validates during CONNECT.
pub(crate) const MNS_TARGET: [u8; 16] = [
    0xbb, 0x58, 0x2b, 0x41, 0x42, 0x0c, 0x11, 0xdb, 0xb0, 0xde, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// MAP-event-report body sent in PUT.
pub(crate) const NEW_MESSAGE_XML: &[u8] = b"<?xml version='1.0'?>\
<MAP-event-report version='1.0'>\
<event type='NewMessage' handle='ABC123' folder='TELECOM/MSG/INBOX' msg_type='SMS_GSM'/>\
</MAP-event-report>";

/// OBEX CONNECT with MNS Target header.
pub(crate) async fn iphone_connect(
    t: &mut obex_core::ObexTransport<DuplexStream>,
) -> Result<(), MnsError> {
    let req = ObexClient::connect_request(&MNS_TARGET).map_err(MnsError::Obex)?;
    t.send(req).await?;
    t.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)?;
    Ok(())
}

/// `PUT_FINAL` with `body` as `EndOfBody`.
pub(crate) async fn iphone_put(
    t: &mut obex_core::ObexTransport<DuplexStream>,
    body: &[u8],
) -> Result<(), MnsError> {
    let pkt = Packet {
        opcode: OpCode::PutFinal,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()
    .map_err(ObexError::Packet)
    .map_err(MnsError::Obex)?;
    t.send(pkt).await?;
    t.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)?;
    Ok(())
}

/// OBEX DISCONNECT.
pub(crate) async fn iphone_disconnect(
    t: &mut obex_core::ObexTransport<DuplexStream>,
) -> Result<(), MnsError> {
    let pkt = Packet { opcode: OpCode::Disconnect, extra: PacketExtra::None, headers: vec![] }
        .encode()
        .map_err(ObexError::Packet)
        .map_err(MnsError::Obex)?;
    t.send(pkt).await?;
    t.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)?;
    Ok(())
}
