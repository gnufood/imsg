//! MNS OBEX server — accepts connections from the remote device and yields MAP event reports.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use obex_core::{packet::OpCode, server::ObexServer};
use obex_core::{wrap, ObexTransport};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::mns_event::{parse_event_report, MnsError, MnsEvent};

const MNS_TARGET: [u8; 16] = [
    0xbb, 0x58, 0x2b, 0x41, 0x42, 0x0c, 0x11, 0xdb, 0xb0, 0xde, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// MNS OBEX server. Receives a connection from the remote device on the MNS RFCOMM channel,
/// performs the OBEX handshake, and yields MAP event reports via [`next_event`](Self::next_event).
///
/// Obtain via [`MnsServer::accept`].
pub struct MnsServer<T> {
    transport: ObexTransport<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> MnsServer<T> {
    /// Wraps `stream` in OBEX framing, performs the OBEX CONNECT handshake as the MNS server,
    /// and validates the `Target` UUID against the MNS service UUID.
    ///
    /// # Errors
    ///
    /// Returns [`MnsError::InvalidTarget`] if the CONNECT request Target header is absent or
    /// does not match `bb582b41-420c-11db-b0de-0800200c9a66`. Returns [`MnsError::Obex`] on
    /// packet decode failure or [`MnsError::Transport`] on I/O failure.
    pub async fn accept(stream: T) -> Result<Self, MnsError> {
        let mut transport = wrap(stream);
        let mut server = ObexServer::new();
        let req = Self::recv(&mut transport).await?;
        let (packet, rsp) = server.handle_connect(&req, &MNS_TARGET)?;
        if packet.header_target() != Some(MNS_TARGET.as_ref()) {
            return Err(MnsError::InvalidTarget);
        }
        transport.send(rsp).await?;
        Ok(Self { transport })
    }

    /// Returns the raw event-report XML body bytes from the next OBEX PUT, or `None` on
    /// OBEX DISCONNECT. Handles the full OBEX exchange. Does not parse the body — callers
    /// needing a typed [`MnsEvent`] should call [`next_event`](Self::next_event) instead.
    ///
    /// # Errors
    ///
    /// Returns [`MnsError::UnexpectedOpcode`] if the device sends an opcode other than PUT,
    /// `PUT_FINAL`, or DISCONNECT. Returns [`MnsError::UnexpectedEof`] if the stream closes
    /// mid-packet. Returns [`MnsError::Transport`] on I/O failure.
    pub async fn next_event_raw(&mut self) -> Result<Option<Bytes>, MnsError> {
        let bytes = Self::recv(&mut self.transport).await?;
        let opcode = bytes.first().copied().ok_or(MnsError::UnexpectedEof)?;
        match OpCode::from_byte(opcode) {
            OpCode::Put | OpCode::PutFinal => {
                let (body, rsp) = ObexServer::handle_put(&bytes)?;
                self.transport.send(rsp).await?;
                Ok(Some(body.unwrap_or_default()))
            }
            OpCode::Disconnect => {
                self.transport.send(ObexServer::ok_response()).await?;
                Ok(None)
            }
            other => {
                let _ = self.transport.send(ObexServer::bad_request_response()).await;
                Err(MnsError::UnexpectedOpcode(other.to_byte()))
            }
        }
    }

    /// Returns `Ok(None)` when the device sends a clean OBEX DISCONNECT.
    ///
    /// # Errors
    ///
    /// Returns [`MnsError::UnexpectedOpcode`] if the device sends an opcode other than PUT,
    /// `PUT_FINAL`, or DISCONNECT. Returns [`MnsError::UnexpectedEof`] if the stream closes
    /// mid-packet. XML parse errors from [`parse_event_report`] are propagated unchanged.
    pub async fn next_event(&mut self) -> Result<Option<MnsEvent>, MnsError> {
        match self.next_event_raw().await? {
            Some(body) => Ok(Some(parse_event_report(&body)?)),
            None => Ok(None),
        }
    }

    async fn recv(transport: &mut ObexTransport<T>) -> Result<Bytes, MnsError> {
        transport.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)
    }
}
