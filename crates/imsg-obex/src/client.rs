use bytes::Bytes;
use thiserror::Error;

use crate::{
    headers::Header,
    packet::{OpCode, Packet, PacketError, PacketExtra},
};

const OBEX_VERSION: u8 = 0x10;
const OBEX_FLAGS: u8 = 0x00;
const OBEX_MAX_PACKET: u16 = 0xFFFF;

// SETPATH 0x02: navigate to child, do not create
const SETPATH_NAVIGATE: u8 = 0x02;
// SETPATH 0x03: navigate to parent (backup bit set, no-create)
const SETPATH_BACKUP: u8 = 0x03;

/// OBEX client errors — connection state, packet codec, server rejection, and missing protocol headers.
#[derive(Debug, Error)]
pub enum ObexError {
    /// No active connection; call `handle_connect_response` first.
    #[error("not connected")]
    NotConnected,
    /// Packet codec failure during request encoding or response decode.
    #[error("packet error: {0}")]
    Packet(#[from] PacketError),
    /// Server refused CONNECT; carries the response opcode byte.
    #[error("connect rejected with opcode {0:#04x}")]
    ConnectRejected(u8),
    /// CONNECT response did not include a `ConnectionId` header; the session cannot be used.
    #[error("connect response missing ConnectionId header")]
    MissingConnectionId,
    /// Message body exceeds the 4 GiB limit the OBEX `Length` header can express.
    #[error("message body too large")]
    BodyTooLarge,
}

enum State {
    Disconnected,
    Connected { conn_id: u32, max_packet: u16 },
}

/// Sans-IO OBEX client state machine. No I/O — callers handle transport.
pub struct ObexClient {
    state: State,
}

impl ObexClient {
    /// Initial state: disconnected.
    #[must_use]
    pub const fn new() -> Self {
        Self { state: State::Disconnected }
    }

    /// Targets the given 16-byte service UUID.
    ///
    /// # Errors
    /// Returns `Packet` if encoding fails (packet too large — not possible in practice).
    pub fn connect_request(target_uuid: &[u8; 16]) -> Result<Bytes, ObexError> {
        Ok(Packet {
            opcode: OpCode::Connect,
            extra: PacketExtra::Connect {
                version: OBEX_VERSION,
                flags: OBEX_FLAGS,
                max_packet: OBEX_MAX_PACKET,
            },
            headers: vec![Header::Target(Bytes::copy_from_slice(target_uuid))],
        }
        .encode()?)
    }

    /// Transitions to Connected on success; returns the assigned connection ID.
    ///
    /// # Errors
    /// Returns `ConnectRejected` if the server returns a non-OK opcode, `MissingConnectionId`
    /// if the response contains no `ConnectionId` header, or `Packet` on decode failure.
    pub fn handle_connect_response(&mut self, data: &[u8]) -> Result<u32, ObexError> {
        let packet = Packet::decode_connect_response(data)?;
        if !packet.opcode.is_ok() {
            return Err(ObexError::ConnectRejected(packet.opcode.to_byte()));
        }
        let conn_id = packet.header_connection_id().ok_or(ObexError::MissingConnectionId)?;
        let max_packet = match &packet.extra {
            PacketExtra::Connect { max_packet, .. } => *max_packet,
            _ => OBEX_MAX_PACKET,
        };
        self.state = State::Connected { conn_id, max_packet };
        Ok(conn_id)
    }

    /// SETPATH navigate-to-child by name.
    ///
    /// # Errors
    /// Returns `NotConnected` if called before a successful CONNECT exchange.
    pub fn setpath_request(&self, name: &str) -> Result<Bytes, ObexError> {
        let conn_id = self.conn_id()?;
        Ok(Packet {
            opcode: OpCode::SetPath,
            extra: PacketExtra::SetPath { flags: SETPATH_NAVIGATE, constants: 0x00 },
            headers: vec![Header::ConnectionId(conn_id), Header::Name(name.to_owned())],
        }
        .encode()?)
    }

    /// SETPATH backup bit, empty Name. Use before navigating to a sibling. Does not validate current depth.
    ///
    /// # Errors
    ///
    /// Returns [`ObexError::NotConnected`] if called before a successful CONNECT exchange.
    pub fn setpath_backup_request(&self) -> Result<Bytes, ObexError> {
        let conn_id = self.conn_id()?;
        Ok(Packet {
            opcode: OpCode::SetPath,
            extra: PacketExtra::SetPath { flags: SETPATH_BACKUP, constants: 0x00 },
            headers: vec![Header::ConnectionId(conn_id), Header::Name(String::new())],
        }
        .encode()?)
    }

    /// GET FINAL with `Type`, optional `Name`, and optional `AppParams`.
    ///
    /// # Errors
    /// Returns `NotConnected` if called before a successful CONNECT exchange.
    pub fn get_request(
        &self,
        type_: &[u8],
        name: Option<&str>,
        app_params: Option<Bytes>,
    ) -> Result<Bytes, ObexError> {
        let conn_id = self.conn_id()?;
        let mut headers =
            vec![Header::ConnectionId(conn_id), Header::Type(Bytes::copy_from_slice(type_))];
        if let Some(n) = name {
            headers.push(Header::Name(n.to_owned()));
        }
        if let Some(params) = app_params {
            headers.push(Header::AppParams(params));
        }
        Ok(Packet { opcode: OpCode::GetFinal, extra: PacketExtra::None, headers }.encode()?)
    }

    /// GET FINAL with only `ConnectionId` — continues a multi-packet exchange after `Continue`.
    ///
    /// # Errors
    /// Returns `NotConnected` if called before a successful CONNECT exchange.
    pub fn get_continue_request(&self) -> Result<Bytes, ObexError> {
        let conn_id = self.conn_id()?;
        Ok(Packet {
            opcode: OpCode::GetFinal,
            extra: PacketExtra::None,
            headers: vec![Header::ConnectionId(conn_id)],
        }
        .encode()?)
    }

    /// Prepends `ConnectionId` and `Type` before `extra_headers`; opcode is always `PutFinal`.
    ///
    /// # Errors
    ///
    /// Returns `NotConnected` if called before a successful `CONNECT` exchange.
    /// Returns `Packet` if encoding fails (not possible in practice for typical header counts).
    pub fn put_final_request(
        &self,
        type_: &[u8],
        extra_headers: Vec<Header>,
    ) -> Result<Bytes, ObexError> {
        let conn_id = self.conn_id()?;
        let mut headers =
            vec![Header::ConnectionId(conn_id), Header::Type(Bytes::copy_from_slice(type_))];
        headers.extend(extra_headers);
        Ok(Packet { opcode: OpCode::PutFinal, extra: PacketExtra::None, headers }.encode()?)
    }

    /// Sends `ConnectionId` in the DISCONNECT payload.
    ///
    /// # Errors
    /// Returns `NotConnected` if called before a successful CONNECT exchange.
    pub fn disconnect_request(&self) -> Result<Bytes, ObexError> {
        let conn_id = self.conn_id()?;
        Ok(Packet {
            opcode: OpCode::Disconnect,
            extra: PacketExtra::None,
            headers: vec![Header::ConnectionId(conn_id)],
        }
        .encode()?)
    }

    /// Stateless decode — does not advance client state.
    ///
    /// # Errors
    /// Returns `Packet` on any decode failure.
    pub fn parse_response(data: &[u8]) -> Result<Packet, ObexError> {
        Ok(Packet::decode(data)?)
    }

    /// Connection ID assigned by the remote in the CONNECT response.
    ///
    /// # Errors
    ///
    /// Returns [`ObexError::NotConnected`] before a successful CONNECT exchange.
    pub const fn conn_id(&self) -> Result<u32, ObexError> {
        match &self.state {
            State::Connected { conn_id, .. } => Ok(*conn_id),
            State::Disconnected => Err(ObexError::NotConnected),
        }
    }

    /// True after a successful CONNECT exchange; false after disconnect or before first connect.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        matches!(self.state, State::Connected { .. })
    }

    /// 0 if not connected; otherwise the server-negotiated max.
    #[must_use]
    pub const fn max_packet(&self) -> u16 {
        match &self.state {
            State::Connected { max_packet, .. } => *max_packet,
            State::Disconnected => 0,
        }
    }
}

impl Default for ObexClient {
    fn default() -> Self {
        Self::new()
    }
}
