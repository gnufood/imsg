use bytes::{BufMut, Bytes, BytesMut};
use thiserror::Error;
use zerocopy::byteorder::network_endian::U16;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::headers::{decode_headers, Header};

// zerocopy derives verify layout safety at compile time.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct FrameHeader {
    opcode: u8,
    length: U16,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct ConnectWire {
    version: u8,
    flags: u8,
    max_packet: U16,
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Unaligned)]
#[repr(C)]
struct SetPathWire {
    flags: u8,
    constants: u8,
}

/// OBEX operation code, carried in the first byte of every packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    /// Initiate a session.
    Connect,
    /// Terminate the session.
    Disconnect,
    /// Send an object, non-final chunk.
    Put,
    /// Send an object, final chunk.
    PutFinal,
    /// Request an object, non-final.
    Get,
    /// Request an object, final request.
    GetFinal,
    /// Navigate the remote folder hierarchy.
    SetPath,
    /// Abort an in-progress multi-packet operation.
    Abort,
    /// Server requests the next chunk.
    Continue,
    /// Success.
    Ok,
    /// Object was created successfully.
    Created,
    /// Request was malformed.
    BadRequest,
    /// Authentication required.
    Unauthorized,
    /// Request understood but refused.
    Forbidden,
    /// Requested object does not exist.
    NotFound,
    /// 0xD0 — unspecified server fault.
    InternalServerError,
    /// Operation not supported by server.
    NotImplemented,
    /// Unrecognized opcode; carries the raw byte.
    Other(u8),
}

impl OpCode {
    /// Parses an opcode byte; unrecognized values become `Other(b)`.
    #[must_use]
    pub const fn from_byte(b: u8) -> Self {
        match b {
            0x80 => Self::Connect,
            0x81 => Self::Disconnect,
            0x02 => Self::Put,
            0x82 => Self::PutFinal,
            0x03 => Self::Get,
            0x83 => Self::GetFinal,
            0x85 => Self::SetPath,
            0xFF => Self::Abort,
            0x90 => Self::Continue,
            0xA0 => Self::Ok,
            0xA1 => Self::Created,
            0xC0 => Self::BadRequest,
            0xC1 => Self::Unauthorized,
            0xC3 => Self::Forbidden,
            0xC4 => Self::NotFound,
            0xD0 => Self::InternalServerError,
            0xD4 => Self::NotImplemented,
            other => Self::Other(other),
        }
    }

    /// Inverse of [`from_byte`]; raw wire octet.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Connect => 0x80,
            Self::Disconnect => 0x81,
            Self::Put => 0x02,
            Self::PutFinal => 0x82,
            Self::Get => 0x03,
            Self::GetFinal => 0x83,
            Self::SetPath => 0x85,
            Self::Abort => 0xFF,
            Self::Continue => 0x90,
            Self::Ok => 0xA0,
            Self::Created => 0xA1,
            Self::BadRequest => 0xC0,
            Self::Unauthorized => 0xC1,
            Self::Forbidden => 0xC3,
            Self::NotFound => 0xC4,
            Self::InternalServerError => 0xD0,
            Self::NotImplemented => 0xD4,
            Self::Other(b) => b,
        }
    }

    /// True for `Ok` and `Created`.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok | Self::Created)
    }

    /// True for `Continue` (0x90).
    #[must_use]
    pub const fn is_continue(self) -> bool {
        matches!(self, Self::Continue)
    }
}

/// Fixed bytes between the length field and the headers, present only for CONNECT and SETPATH.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketExtra {
    /// No extra fixed bytes; applies to most opcodes.
    None,
    /// CONNECT fixed header: version, flags, and negotiated max packet size.
    Connect {
        /// OBEX version byte; 0x10 = v1.0.
        version: u8,
        /// Reserved; always 0x00.
        flags: u8,
        /// Maximum packet size the sender accepts.
        max_packet: u16,
    },
    /// SETPATH fixed header: navigation flags.
    SetPath {
        /// Bit 1 = navigate to child; bit 0 = do not create.
        flags: u8,
        /// Reserved; always 0x00.
        constants: u8,
    },
}

/// Decoded packet: opcode, optional fixed-byte section, and variable headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    /// First wire byte, decoded.
    pub opcode: OpCode,
    /// Fixed bytes between the length field and the headers.
    pub extra: PacketExtra,
    /// Variable-length headers in wire order.
    pub headers: Vec<Header>,
}

/// Packet codec errors — malformed framing, invalid headers, UTF-16 name decode, and size limits.
#[derive(Debug, Error)]
pub enum PacketError {
    /// Data ends before the 3-byte frame header or declared packet length.
    #[error("packet too short")]
    TooShort,
    /// Header HI byte or length encoding is malformed.
    #[error("invalid header encoding")]
    InvalidHeader,
    /// Name header contains invalid UTF-16.
    #[error("invalid UTF-16 in name header")]
    InvalidName,
    /// Encoded packet length exceeds the OBEX 65535-byte limit.
    #[error("packet too large to encode")]
    PacketTooLarge,
    /// Single header payload exceeds the 65532-byte per-header limit.
    #[error("header payload too large to encode")]
    HeaderTooLarge,
}

impl Packet {
    /// Opcode determines which `PacketExtra` variant to read.
    ///
    /// # Errors
    /// Returns `TooShort` if `data` is shorter than the declared length, or `InvalidHeader`
    /// if any header tag or length encoding is malformed.
    pub fn decode(data: &[u8]) -> Result<Self, PacketError> {
        parse_packet(data)
    }

    /// Always parses 4 extra bytes before headers regardless of opcode — CONNECT responses require this.
    ///
    /// # Errors
    /// Returns `TooShort` if `data` is shorter than the declared length, or `InvalidHeader`
    /// if any header is malformed.
    pub fn decode_connect_response(data: &[u8]) -> Result<Self, PacketError> {
        parse_connect_response(data)
    }

    /// Fails if total length exceeds 65535 bytes (OBEX packet limit).
    ///
    /// # Errors
    /// Returns `PacketTooLarge` if total length overflows u16, or `HeaderTooLarge` if any
    /// individual header payload exceeds 65532 bytes.
    #[must_use = "encoded bytes must be sent"]
    pub fn encode(&self) -> Result<Bytes, PacketError> {
        let mut body = BytesMut::with_capacity(256);
        match &self.extra {
            PacketExtra::None => {}
            PacketExtra::Connect { version, flags, max_packet } => {
                body.put_slice(
                    ConnectWire {
                        version: *version,
                        flags: *flags,
                        max_packet: U16::new(*max_packet),
                    }
                    .as_bytes(),
                );
            }
            PacketExtra::SetPath { flags, constants } => {
                body.put_slice(SetPathWire { flags: *flags, constants: *constants }.as_bytes());
            }
        }
        for h in &self.headers {
            h.encode_into(&mut body)?;
        }
        let total = body
            .len()
            .checked_add(3)
            .and_then(|n| u16::try_from(n).ok())
            .ok_or(PacketError::PacketTooLarge)?;
        let mut out = BytesMut::with_capacity(total.into());
        out.put_slice(
            FrameHeader { opcode: self.opcode.to_byte(), length: U16::new(total) }.as_bytes(),
        );
        out.put(body);
        Ok(out.freeze())
    }

    /// First `ConnectionId` header value, if present.
    #[must_use]
    pub fn header_connection_id(&self) -> Option<u32> {
        self.headers.iter().find_map(Header::connection_id)
    }

    /// First `Target` header payload, if present.
    #[must_use]
    pub fn header_target(&self) -> Option<&[u8]> {
        self.headers.iter().find_map(|h| {
            if let Header::Target(b) = h {
                Some(b.as_ref())
            } else {
                None
            }
        })
    }

    /// First non-empty Name header string, if present. Wire UTF-16BE is decoded before return.
    #[must_use]
    pub fn header_name(&self) -> Option<String> {
        self.headers.iter().find_map(|h| match h {
            Header::Name(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
    }

    /// First `Body` or `EndOfBody` payload, if present.
    #[must_use]
    pub fn body_payload(&self) -> Option<&[u8]> {
        self.headers.iter().find_map(|h| match h {
            Header::EndOfBody(b) | Header::Body(b) => Some(b.as_ref()),
            _ => None,
        })
    }
}

fn framed(data: &[u8]) -> Result<(OpCode, &[u8]), PacketError> {
    let (hdr, rest) = FrameHeader::ref_from_prefix(data).map_err(|_| PacketError::TooShort)?;
    let total: usize = hdr.length.get().into();
    let body_len = total.checked_sub(3).ok_or(PacketError::TooShort)?;
    let body = rest.get(..body_len).ok_or(PacketError::TooShort)?;
    Ok((OpCode::from_byte(hdr.opcode), body))
}

fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    let (opcode, body) = framed(data)?;
    let (extra, headers_bytes) = match opcode {
        OpCode::Connect => {
            let (w, rest) =
                ConnectWire::ref_from_prefix(body).map_err(|_| PacketError::TooShort)?;
            (
                PacketExtra::Connect {
                    version: w.version,
                    flags: w.flags,
                    max_packet: w.max_packet.get(),
                },
                rest,
            )
        }
        OpCode::SetPath => {
            let (w, rest) =
                SetPathWire::ref_from_prefix(body).map_err(|_| PacketError::TooShort)?;
            (PacketExtra::SetPath { flags: w.flags, constants: w.constants }, rest)
        }
        _ => (PacketExtra::None, body),
    };
    let mut input = headers_bytes;
    let headers = decode_headers(&mut input)?;
    Ok(Packet { opcode, extra, headers })
}

fn parse_connect_response(data: &[u8]) -> Result<Packet, PacketError> {
    let (opcode, body) = framed(data)?;
    let (w, rest) = ConnectWire::ref_from_prefix(body).map_err(|_| PacketError::TooShort)?;
    let extra =
        PacketExtra::Connect { version: w.version, flags: w.flags, max_packet: w.max_packet.get() };
    let mut input = rest;
    let headers = decode_headers(&mut input)?;
    Ok(Packet { opcode, extra, headers })
}
