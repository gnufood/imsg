//! Message CRUD: listing, fetch, send, and status updates.

use bytes::Bytes;
use formats::bmessage::BMessage;
use futures::SinkExt;
use obex_core::{
    client::{ObexClient, ObexError},
    headers::Header,
};
use tokio::io::{AsyncRead, AsyncWrite};

use super::MapClient;
use crate::{
    messages::{ListMessagesFilter, MessageEntry},
    params::{
        get_message_params, push_message_params, set_message_status_params,
        INDICATOR_DELETED_STATUS, INDICATOR_READ_STATUS,
    },
    MapError, MessageStatus,
};

impl<T: AsyncRead + AsyncWrite + Unpin> MapClient<T> {
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
}
