use bytes::{BufMut, Bytes, BytesMut};
use winnow::{
    binary::{be_u16, be_u32, be_u8},
    combinator::repeat,
    error::ContextError,
    token::take,
    Parser,
};

use crate::packet::PacketError;

const HDR_NAME: u8 = 0x01;
const HDR_TYPE: u8 = 0x42;
const HDR_LENGTH: u8 = 0xC3;
const HDR_TARGET: u8 = 0x46;
const HDR_WHO: u8 = 0x4A;
const HDR_BODY: u8 = 0x48;
const HDR_END_OF_BODY: u8 = 0x49;
const HDR_APP_PARAMS: u8 = 0x4C;
const HDR_CONN_ID: u8 = 0xCB;
const HDR_SRM: u8 = 0x97;

/// An OBEX header decoded from or encoded into a packet body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Header {
    /// Empty name = 3-byte header with no payload; non-empty carries the folder/object name as UTF-16BE.
    Name(String),
    /// Object type string, null-terminated ASCII (e.g. `x-bt/message\0`).
    Type(Bytes),
    /// Pre-declared object size; not validated against actual payload by this codec.
    Length(u32),
    /// 16-byte service UUID identifying the target profile.
    Target(Bytes),
    /// 16-byte UUID echoed by the server identifying the responding profile.
    Who(Bytes),
    /// Intermediate chunk of a multi-packet body.
    Body(Bytes),
    /// Final (or only) chunk of the object body.
    EndOfBody(Bytes),
    /// Profile-specific application parameters blob.
    AppParams(Bytes),
    /// Connection handle; must be echoed in all subsequent requests after CONNECT.
    ConnectionId(u32),
    /// Single Response Mode control byte; 1 = enabled.
    Srm(u8),
    /// Unrecognized header: tag byte and raw value.
    Unknown(u8, Bytes),
}

impl Header {
    // returns HeaderTooLarge if payload + 3-byte overhead exceeds u16::MAX
    pub(crate) fn encode_into(&self, buf: &mut BytesMut) -> Result<(), PacketError> {
        match self {
            Self::Name(s) => {
                if s.is_empty() {
                    buf.put_u8(HDR_NAME);
                    buf.put_u16(3);
                } else {
                    let encoded: Vec<u8> =
                        s.encode_utf16().flat_map(u16::to_be_bytes).chain([0u8, 0u8]).collect();
                    encode_byteseq(HDR_NAME, &encoded, buf)?;
                }
            }
            Self::Type(data) => encode_byteseq(HDR_TYPE, data, buf)?,
            Self::Length(v) => encode_4byte(HDR_LENGTH, *v, buf),
            Self::Target(data) => encode_byteseq(HDR_TARGET, data, buf)?,
            Self::Who(data) => encode_byteseq(HDR_WHO, data, buf)?,
            Self::Body(data) => encode_byteseq(HDR_BODY, data, buf)?,
            Self::EndOfBody(data) => encode_byteseq(HDR_END_OF_BODY, data, buf)?,
            Self::AppParams(data) => encode_byteseq(HDR_APP_PARAMS, data, buf)?,
            Self::ConnectionId(v) => encode_4byte(HDR_CONN_ID, *v, buf),
            Self::Srm(v) => {
                buf.put_u8(HDR_SRM);
                buf.put_u8(*v);
            }
            Self::Unknown(id, data) => encode_byteseq(*id, data, buf)?,
        }
        Ok(())
    }

    #[must_use]
    pub(crate) const fn connection_id(&self) -> Option<u32> {
        if let Self::ConnectionId(id) = self {
            Some(*id)
        } else {
            None
        }
    }
}

fn encode_byteseq(id: u8, data: &[u8], buf: &mut BytesMut) -> Result<(), PacketError> {
    let len = data
        .len()
        .checked_add(3)
        .and_then(|n| u16::try_from(n).ok())
        .ok_or(PacketError::HeaderTooLarge)?;
    buf.put_u8(id);
    buf.put_u16(len);
    buf.put_slice(data);
    Ok(())
}

fn encode_4byte(id: u8, val: u32, buf: &mut BytesMut) {
    buf.put_u8(id);
    buf.put_u32(val);
}

// advances input to its end
pub(crate) fn decode_headers(input: &mut &[u8]) -> Result<Vec<Header>, PacketError> {
    let mut headers = Vec::new();
    while !input.is_empty() {
        headers.push(decode_one(input)?);
    }
    Ok(headers)
}

fn decode_name_str(data: &[u8]) -> Result<String, PacketError> {
    if data.is_empty() {
        return Ok(String::new());
    }
    if data.len() % 2 != 0 {
        return Err(PacketError::InvalidName);
    }
    let body_len = data.len().checked_sub(2).ok_or(PacketError::InvalidName)?;
    let mut input = data;
    let without_null = take(body_len)
        .parse_next(&mut input)
        .map_err(|_: ContextError| PacketError::InvalidName)?;
    let mut u16_input = without_null;
    let utf16: Vec<u16> = repeat(0.., be_u16)
        .parse_next(&mut u16_input)
        .map_err(|_: ContextError| PacketError::InvalidName)?;
    String::from_utf16(&utf16).map_err(|_| PacketError::InvalidName)
}

fn decode_one(input: &mut &[u8]) -> Result<Header, PacketError> {
    let id = be_u8(input).map_err(|_: ContextError| PacketError::InvalidHeader)?;
    let kind = (id >> 6) & 0x03;
    let header = match kind {
        0 => {
            let hlen: usize =
                be_u16(input).map_err(|_: ContextError| PacketError::InvalidHeader)?.into();
            if hlen < 3 {
                return Err(PacketError::InvalidHeader);
            }
            let body_len = hlen.checked_sub(3).ok_or(PacketError::InvalidHeader)?;
            let data = take(body_len)
                .parse_next(input)
                .map_err(|_: ContextError| PacketError::InvalidHeader)?;
            Header::Name(decode_name_str(data)?)
        }
        1 => {
            let hlen: usize =
                be_u16(input).map_err(|_: ContextError| PacketError::InvalidHeader)?.into();
            if hlen < 3 {
                return Err(PacketError::InvalidHeader);
            }
            let body_len = hlen.checked_sub(3).ok_or(PacketError::InvalidHeader)?;
            let data = take(body_len)
                .parse_next(input)
                .map_err(|_: ContextError| PacketError::InvalidHeader)?;
            let bytes = Bytes::copy_from_slice(data);
            match id {
                HDR_TYPE => Header::Type(bytes),
                HDR_TARGET => Header::Target(bytes),
                HDR_WHO => Header::Who(bytes),
                HDR_BODY => Header::Body(bytes),
                HDR_END_OF_BODY => Header::EndOfBody(bytes),
                HDR_APP_PARAMS => Header::AppParams(bytes),
                _ => Header::Unknown(id, bytes),
            }
        }
        2 => {
            let val = be_u8(input).map_err(|_: ContextError| PacketError::InvalidHeader)?;
            match id {
                HDR_SRM => Header::Srm(val),
                _ => Header::Unknown(id, Bytes::copy_from_slice(&[val])),
            }
        }
        3 => {
            let val = be_u32(input).map_err(|_: ContextError| PacketError::InvalidHeader)?;
            match id {
                HDR_CONN_ID => Header::ConnectionId(val),
                HDR_LENGTH => Header::Length(val),
                _ => Header::Unknown(id, Bytes::copy_from_slice(&val.to_be_bytes())),
            }
        }
        _ => unreachable!(),
    };
    Ok(header)
}
