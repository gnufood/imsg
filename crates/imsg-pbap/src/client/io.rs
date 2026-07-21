//! Low-level OBEX response collection: CONTINUE-loop draining, body accumulation, and the
//! shared frame-receive helper. Split out of `mod.rs` to keep PBAP operations (pull/list/search)
//! separate from response-framing plumbing.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use obex_core::client::ObexClient;
use obex_core::ObexTransport;
use tokio::io::{AsyncRead, AsyncWrite};

use super::PbapClient;
use crate::PbapError;

const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

impl<T: AsyncRead + AsyncWrite + Unpin> PbapClient<T> {
    pub(super) async fn collect_body(&mut self) -> Result<Vec<u8>, PbapError> {
        Ok(self.collect_response().await?.0)
    }

    /// Drives the CONTINUE loop to completion, returning the accumulated body and the last
    /// non-empty `AppParams` header seen (a metadata-only response has no body; a body response
    /// may carry no `AppParams` at all).
    pub(super) async fn collect_response(&mut self) -> Result<(Vec<u8>, Option<Bytes>), PbapError> {
        let mut body = Vec::with_capacity(4096);
        let mut app_params = None;
        loop {
            let rsp_bytes = Self::recv(&mut self.transport).await?;
            let rsp = ObexClient::parse_response(&rsp_bytes)?;
            if let Some(params) = rsp.header_app_params() {
                app_params = Some(Bytes::copy_from_slice(params));
            }
            if rsp.opcode.is_continue() {
                Self::append_chunk(&mut body, rsp.body_payload())?;
                let cont = self.obex.get_continue_request()?;
                self.transport.send(cont).await?;
            } else if rsp.opcode.is_ok() {
                Self::append_chunk(&mut body, rsp.body_payload())?;
                break;
            } else {
                return Err(PbapError::ServerError(rsp.opcode.to_byte()));
            }
        }
        Ok((body, app_params))
    }

    fn append_chunk(body: &mut Vec<u8>, chunk: Option<&[u8]>) -> Result<(), PbapError> {
        let Some(chunk) = chunk else { return Ok(()) };
        let new_len = body.len().checked_add(chunk.len()).ok_or(PbapError::ResponseTooLarge)?;
        if new_len > MAX_BODY_BYTES {
            return Err(PbapError::ResponseTooLarge);
        }
        body.extend_from_slice(chunk);
        Ok(())
    }

    pub(super) async fn recv(transport: &mut ObexTransport<T>) -> Result<Bytes, PbapError> {
        transport.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)
    }
}
