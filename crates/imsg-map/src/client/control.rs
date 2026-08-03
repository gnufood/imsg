//! Folder listing, notification registration, and session teardown.

use bytes::Bytes;
use formats::xml::FolderListing;
use futures::{SinkExt, StreamExt};
use obex_core::{client::ObexClient, headers::Header};
use tokio::io::{AsyncRead, AsyncWrite};

use super::MapClient;
use crate::{params::set_notification_registration_params, MapError};

impl<T: AsyncRead + AsyncWrite + Unpin> MapClient<T> {
    /// Returns the MAP folder listing for the current object store level via `GetFolderListing`
    /// GET with Type `x-obex/folder-listing`. Folders are in device-reported document order.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`MapError::FolderListing`] if the response body is malformed XML.
    /// Returns [`MapError::Obex`] or [`MapError::Transport`] on lower-layer failure.
    pub async fn get_folder_listing(&mut self) -> Result<FolderListing, MapError> {
        let req = self.obex.get_request(b"x-obex/folder-listing\x00", None, None)?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        Ok(FolderListing::parse(&body)?)
    }

    /// When `enable` is `true`, the phone will connect to the MNS channel and push event reports.
    /// When `false`, it stops. The caller must keep the MAP session alive while notifications are active.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`MapError::Obex`] or [`MapError::Transport`] on lower-layer failure.
    pub async fn set_notification_registration(&mut self, enable: bool) -> Result<(), MapError> {
        let req = self.obex.put_final_request(
            b"x-bt/MAP-NotificationRegistration\x00",
            vec![
                Header::AppParams(set_notification_registration_params(enable)),
                Header::EndOfBody(Bytes::new()),
            ],
        )?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        Ok(())
    }

    /// Reads from the transport until the remote closes the stream, discarding all received
    /// packets. Returns `Ok(())` on clean close. Does not send OBEX DISCONNECT and does not
    /// parse received packet opcodes.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::Transport`] on a framing error from the underlying codec.
    pub async fn hold(&mut self) -> Result<(), MapError> {
        loop {
            match self.transport.next().await {
                None => return Ok(()),
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(MapError::Transport(e)),
            }
        }
    }

    /// Sends OBEX DISCONNECT and awaits the server acknowledgement.
    ///
    /// Consumes `self` — the session is unusable after this call regardless of the outcome.
    /// Does not close the underlying stream; the stream is dropped when `self` is consumed.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the request cannot be encoded, the transport fails, or the
    /// server returns a non-OK response.
    pub async fn disconnect(mut self) -> Result<(), MapError> {
        let req = self.obex.disconnect_request()?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        Ok(())
    }
}
