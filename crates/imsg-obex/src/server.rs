//! Sans-IO OBEX server state machine. Symmetric counterpart to [`client::ObexClient`].

use bytes::Bytes;

use crate::{
    client::ObexError,
    headers::Header,
    packet::{OpCode, Packet, PacketExtra},
};

const OBEX_VERSION: u8 = 0x10;
const OBEX_FLAGS: u8 = 0x00;
const OBEX_MAX_PACKET: u16 = 0xFFFF;

/// Issues monotonically increasing connection IDs; callers handle transport.
pub struct ObexServer {
    next_conn_id: u32,
}

impl ObexServer {
    /// First connection ID is `1`.
    #[must_use]
    pub const fn new() -> Self {
        Self { next_conn_id: 1 }
    }

    /// Does not validate the Target UUID — call [`Packet::header_target`] before sending the response.
    ///
    /// # Errors
    ///
    /// Returns [`ObexError::Packet`] if `data` cannot be decoded as a valid OBEX packet.
    pub fn handle_connect(
        &mut self,
        data: &[u8],
        who_uuid: &[u8; 16],
    ) -> Result<(Packet, Bytes), ObexError> {
        let packet = Packet::decode(data)?;
        let conn_id = self.next_conn_id;
        self.next_conn_id = self.next_conn_id.saturating_add(1);
        let rsp = Packet {
            opcode: OpCode::Ok,
            extra: PacketExtra::Connect {
                version: OBEX_VERSION,
                flags: OBEX_FLAGS,
                max_packet: OBEX_MAX_PACKET,
            },
            headers: vec![
                Header::ConnectionId(conn_id),
                Header::Who(Bytes::copy_from_slice(who_uuid)),
            ],
        }
        .encode()?;
        Ok((packet, rsp))
    }

    /// Returns `(None, ok)` if no `Body`/`EndOfBody` header. Opcode is not validated — caller must ensure PUT or `PUT_FINAL`.
    ///
    /// # Errors
    ///
    /// Returns [`ObexError::Packet`] if `data` cannot be decoded as a valid OBEX packet.
    pub fn handle_put(data: &[u8]) -> Result<(Option<Bytes>, Bytes), ObexError> {
        let packet = Packet::decode(data)?;
        let body = packet.body_payload().map(Bytes::copy_from_slice);
        Ok((body, Self::ok_response()))
    }

    /// `[0xA0, 0x00, 0x03]` — response to PUT and DISCONNECT.
    #[must_use]
    pub const fn ok_response() -> Bytes {
        Bytes::from_static(&[0xA0, 0x00, 0x03])
    }

    /// `[0xC0, 0x00, 0x03]` — send before returning error on unknown opcodes, or the remote will hang.
    #[must_use]
    pub const fn bad_request_response() -> Bytes {
        Bytes::from_static(&[0xC0, 0x00, 0x03])
    }
}

impl Default for ObexServer {
    fn default() -> Self {
        Self::new()
    }
}
