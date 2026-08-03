//! MAP client — OBEX session setup, SETPATH sequencing, and request dispatch.
//!
//! Split across `nav` (folder navigation), `messages` (message CRUD), and `control`
//! (folder listing, notifications, session teardown); session setup and the shared
//! `recv`/`collect_body` primitives live here.

mod control;
mod messages;
mod nav;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use obex_core::client::ObexClient;
use obex_core::{wrap, ObexTransport};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::MapError;

const MAP_UUID: [u8; 16] = [
    0xbb, 0x58, 0x2b, 0x40, 0x42, 0x0c, 0x11, 0xdb, 0xb0, 0xde, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// MAP session over an OBEX transport. Owns the OBEX state machine and the framed I/O.
///
/// Obtain via [`MapClient::connect`].
pub struct MapClient<T> {
    obex: ObexClient,
    transport: ObexTransport<T>,
    // SETPATH depth confirmed by server; 0 = root, max 3 (telecom/msg/<folder>).
    depth: u8,
}

impl<T: AsyncRead + AsyncWrite + Unpin> MapClient<T> {
    /// Sends the MAP UUID as the OBEX `Target` header and validates the server's response.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the transport fails, packet encoding fails, the server rejects
    /// the connection, or the response omits the `ConnectionId` header.
    pub async fn connect(stream: T) -> Result<Self, MapError> {
        let mut transport = wrap(stream);
        let mut obex = ObexClient::new();
        let req = ObexClient::connect_request(&MAP_UUID, None)?;
        transport.send(req).await?;
        let rsp = Self::recv(&mut transport).await?;
        obex.handle_connect_response(&rsp)?;
        Ok(Self { obex, transport, depth: 0 })
    }

    async fn recv(transport: &mut ObexTransport<T>) -> Result<Bytes, MapError> {
        transport.next().await.ok_or(MapError::UnexpectedEof)?.map_err(MapError::Transport)
    }

    async fn collect_body(&mut self) -> Result<Vec<u8>, MapError> {
        let mut body = Vec::with_capacity(512);
        loop {
            let rsp_bytes = Self::recv(&mut self.transport).await?;
            let rsp = ObexClient::parse_response(&rsp_bytes)?;
            if rsp.opcode.is_continue() {
                if let Some(chunk) = rsp.body_payload() {
                    let new_len =
                        body.len().checked_add(chunk.len()).ok_or(MapError::ResponseTooLarge)?;
                    if new_len > MAX_BODY_BYTES {
                        return Err(MapError::ResponseTooLarge);
                    }
                    body.extend_from_slice(chunk);
                }
                let cont = self.obex.get_continue_request()?;
                self.transport.send(cont).await?;
            } else if rsp.opcode.is_ok() {
                if let Some(chunk) = rsp.body_payload() {
                    let new_len =
                        body.len().checked_add(chunk.len()).ok_or(MapError::ResponseTooLarge)?;
                    if new_len > MAX_BODY_BYTES {
                        return Err(MapError::ResponseTooLarge);
                    }
                    body.extend_from_slice(chunk);
                }
                break;
            } else {
                return Err(MapError::ServerError(rsp.opcode.to_byte()));
            }
        }
        Ok(body)
    }
}
