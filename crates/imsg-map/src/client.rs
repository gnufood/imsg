//! MAP client — OBEX session setup, SETPATH sequencing, and request dispatch.

use bytes::Bytes;
use formats::bmessage::BMessage;
use formats::xml::FolderListing;
use futures::{SinkExt, StreamExt};
use obex_core::{
    client::{ObexClient, ObexError},
    headers::Header,
};
use obex_core::{wrap, ObexTransport};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    folders::Folder,
    messages::{ListMessagesFilter, MessageEntry},
    params::{
        get_message_params, push_message_params, set_message_status_params,
        set_notification_registration_params, INDICATOR_DELETED_STATUS, INDICATOR_READ_STATUS,
    },
    MapError, MessageStatus,
};

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
        let req = ObexClient::connect_request(&MAP_UUID)?;
        transport.send(req).await?;
        let rsp = Self::recv(&mut transport).await?;
        obex.handle_connect_response(&rsp)?;
        Ok(Self { obex, transport, depth: 0 })
    }

    /// `segment` is a single path component, not a slash-joined path — iOS requires one
    /// SETPATH per level. Does not validate that `segment` names an existing folder.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the request fails to encode, the transport closes, or the
    /// server returns a non-OK response.
    pub(crate) async fn setpath(&mut self, segment: &str) -> Result<(), MapError> {
        let req = self.obex.setpath_request(segment)?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        self.depth = self.depth.saturating_add(1);
        Ok(())
    }

    /// Decrements `depth` on success. Does not check whether depth is already zero.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the request fails to encode, the transport closes, or the
    /// server returns a non-OK response.
    pub(crate) async fn setpath_up(&mut self) -> Result<(), MapError> {
        debug_assert!(self.depth > 0, "setpath_up called at root");
        let req = self.obex.setpath_backup_request()?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }

    /// No-op when already at root (`depth == 0`). Does not navigate anywhere after resetting.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if any backup SETPATH fails to encode, the transport closes, or the
    /// server returns a non-OK response.
    pub(crate) async fn reset_to_root(&mut self) -> Result<(), MapError> {
        for _ in 0..self.depth {
            self.setpath_up().await?;
        }
        Ok(())
    }

    /// If already inside a subfolder (e.g. after a prior `set_folder` call), backs up to root
    /// first, then navigates `telecom` → `msg` → folder. iOS requires one SETPATH per level;
    /// a single slash-joined path is rejected.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if any SETPATH fails to encode, the transport closes, or the
    /// server returns a non-OK response for any step.
    pub async fn set_folder(&mut self, folder: Folder) -> Result<(), MapError> {
        self.reset_to_root().await?;
        for segment in ["telecom", "msg", folder.as_str()] {
            self.setpath(segment).await?;
        }
        Ok(())
    }

    /// Caller must navigate to the target folder via [`Self::set_folder`] before calling this.
    /// Sends a `GetMessagesListing` GET with no Name header — the device lists the current OBEX
    /// working directory. Accumulates body chunks across CONTINUE responses before parsing.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the transport fails, the server rejects the request, or the
    /// response XML is malformed.
    pub async fn list_messages(
        &mut self,
        filter: &ListMessagesFilter,
    ) -> Result<Vec<MessageEntry>, MapError> {
        let req = self.obex.get_request(
            b"x-bt/MAP-msg-listing\x00",
            None,
            Some(filter.to_app_params()?),
        )?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        Ok(crate::xml::parse_message_listing(&body)?)
    }

    /// Sends a `GetMessage` GET with `Type: x-bt/message` and `Charset=UTF-8`. Accumulates body
    /// chunks across CONTINUE responses before parsing.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::InvalidInput`] if `handle` is empty or contains CR or LF — it lands
    /// verbatim in the OBEX Name header. Returns [`MapError`] if the transport fails, the server
    /// rejects the request, the response body exceeds 4 MiB, is not valid UTF-8, or the bMessage
    /// is malformed.
    pub async fn get_message(&mut self, handle: &str) -> Result<BMessage, MapError> {
        if handle.is_empty() {
            return Err(MapError::InvalidInput("handle must not be empty"));
        }
        if handle.contains(['\r', '\n']) {
            return Err(MapError::InvalidInput("handle must not contain CR or LF"));
        }
        let req =
            self.obex.get_request(b"x-bt/message\x00", Some(handle), Some(get_message_params()))?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        let text = std::str::from_utf8(&body).map_err(|_| MapError::InvalidEncoding)?;
        Ok(BMessage::parse(text)?)
    }

    /// Sends `text` as an outbound SMS to `phone` via MAP `PushMessage`. Returns the opaque
    /// handle assigned by the remote, suitable for passing to `set_message_status`. Caller must
    /// have navigated to outbox via `set_folder(Folder::Outbox)` first.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if encoding fails, the transport closes, the server rejects the
    /// request, or the OK response contains no Name header.
    pub async fn push_message(&mut self, phone: &str, text: &str) -> Result<String, MapError> {
        if phone.contains(['\r', '\n']) {
            return Err(MapError::InvalidInput("phone must not contain CR or LF"));
        }
        let body = BMessage::outbound_sms(phone, text).encode();
        let body_bytes = body.as_bytes();
        let len = u32::try_from(body_bytes.len()).map_err(|_| ObexError::BodyTooLarge)?;
        let req = self.obex.put_final_request(
            b"x-bt/message\x00",
            vec![
                Header::Name(String::new()),
                Header::Length(len),
                Header::AppParams(push_message_params()),
                Header::EndOfBody(Bytes::copy_from_slice(body_bytes)),
            ],
        )?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        rsp.header_name().ok_or(MapError::MissingHandle)
    }

    /// Marks the message identified by `handle` as `status` (read or unread) via MAP
    /// `SetMessageStatus`. Caller must navigate to the containing folder via `set_folder` first.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::InvalidInput`] if `handle` is empty or contains CR or LF.
    /// Returns [`MapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`MapError::Obex`] or [`MapError::Transport`] on lower-layer failure.
    pub async fn set_message_status_read(
        &mut self,
        handle: &str,
        status: MessageStatus,
    ) -> Result<(), MapError> {
        let value = match status {
            MessageStatus::Read => 0x01_u8,
            MessageStatus::Unread => 0x00_u8,
        };
        self.do_set_message_status(handle, INDICATOR_READ_STATUS, value).await
    }

    /// Marks the message identified by `handle` as deleted (`true`) or undeleted (`false`) via
    /// MAP `SetMessageStatus`. Caller must navigate to the containing folder via `set_folder` first.
    ///
    /// # Errors
    ///
    /// Returns [`MapError::InvalidInput`] if `handle` is empty or contains CR or LF.
    /// Returns [`MapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`MapError::Obex`] or [`MapError::Transport`] on lower-layer failure.
    pub async fn set_message_status_deleted(
        &mut self,
        handle: &str,
        deleted: bool,
    ) -> Result<(), MapError> {
        self.do_set_message_status(handle, INDICATOR_DELETED_STATUS, u8::from(deleted)).await
    }

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

    async fn do_set_message_status(
        &mut self,
        handle: &str,
        indicator: u8,
        value: u8,
    ) -> Result<(), MapError> {
        if handle.is_empty() {
            return Err(MapError::InvalidInput("handle must not be empty"));
        }
        if handle.contains(['\r', '\n']) {
            return Err(MapError::InvalidInput("handle must not contain CR or LF"));
        }
        let req = self.obex.put_final_request(
            b"x-bt/messageStatus\x00",
            vec![
                Header::Name(handle.to_owned()),
                Header::AppParams(Bytes::from(set_message_status_params(indicator, value))),
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

    async fn recv(transport: &mut ObexTransport<T>) -> Result<Bytes, MapError> {
        transport.next().await.ok_or(MapError::UnexpectedEof)?.map_err(MapError::Transport)
    }
}
