//! Sans-IO OBEX packet codec, framing, and client/server state machines.

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;

/// OBEX client state machine — encodes requests and processes responses.
pub mod client;
/// OBEX length-prefix framing codec for [`tokio_util::codec::Framed`].
pub mod codec;
/// OBEX header types; wire tag bytes 0x01–0xCB.
pub mod headers;
/// [`Packet`], [`OpCode`], `PacketExtra`, and `PacketError` types.
pub mod packet;
/// OBEX server state machine — decodes requests and encodes responses.
pub mod server;

pub use codec::ObexCodec;

/// OBEX framing errors — invalid length, unexpected EOF, and external transport failures.
#[derive(Debug, Error)]
pub enum TransportError {
    /// OS-level socket error; check `kind()` for connection-refused, permission-denied, etc.
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Declared packet length is below the 3-byte OBEX minimum.
    #[error("declared packet length {declared} is below the 3-byte OBEX minimum")]
    InvalidLength {
        /// The value read from wire bytes 1–2.
        declared: usize,
    },
    /// Stream closed before a complete packet arrived.
    #[error("unexpected end of stream")]
    UnexpectedEof,
    /// External transport driver error (QUIC, TLS, or other); carries the driver error message.
    #[error("transport driver: {0}")]
    External(String),
}

/// OBEX-framed async transport over any [`AsyncRead`] + [`AsyncWrite`] stream.
///
/// Yields and accepts complete OBEX packets as [`bytes::Bytes`]. Obtain via
/// [`wrap`]; use [`futures::SinkExt`] / [`futures::StreamExt`] to send and receive.
pub type ObexTransport<T> = Framed<T, ObexCodec>;

/// Buffers reads until a complete OBEX packet arrives.
pub fn wrap<T: AsyncRead + AsyncWrite>(inner: T) -> ObexTransport<T> {
    Framed::new(inner, ObexCodec)
}
