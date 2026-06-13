//! OBEX length-prefix framing codec for [`tokio_util::codec::Framed`].

use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::TransportError;

/// Decodes a byte stream into complete OBEX packets by reading the big-endian
/// `u16` total-length field at wire bytes 1–2. Encodes by writing packets as-is
/// (they already carry their own length field).
#[derive(Debug, Clone, Copy, Default)]
pub struct ObexCodec;

impl Decoder for ObexCodec {
    type Item = Bytes;
    type Error = TransportError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 3 {
            return Ok(None);
        }
        let length_bytes: [u8; 2] =
            src.get(1..3).and_then(|s| s.try_into().ok()).ok_or(TransportError::UnexpectedEof)?;
        let declared = usize::from(u16::from_be_bytes(length_bytes));
        if declared < 3 {
            return Err(TransportError::InvalidLength { declared });
        }
        if src.len() < declared {
            src.reserve(declared.saturating_sub(src.len()));
            return Ok(None);
        }
        Ok(Some(src.split_to(declared).freeze()))
    }
}

impl Encoder<Bytes> for ObexCodec {
    type Error = TransportError;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&item);
        Ok(())
    }
}
