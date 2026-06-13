//! PBAP client state machine — session setup and phonebook pull requests.

use bytes::Bytes;
use formats::vcard::Contact;
use futures::{SinkExt, StreamExt};
use obex_core::client::ObexClient;
use obex_core::{wrap, ObexTransport};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    contacts::{normalize_number, parse_card_listing, parse_contacts, CardEntry},
    params::{pull_all_params, pull_entry_params},
    phonebook::PhonebookPath,
    PbapError,
};

const PBAP_UUID: [u8; 16] = [
    0x79, 0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// Owns the OBEX state machine and framed I/O. Obtain via [`connect`](Self::connect).
pub struct PbapClient<T> {
    obex: ObexClient,
    transport: ObexTransport<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> PbapClient<T> {
    /// Sends PBAP PSE UUID as OBEX `Target` and validates the server response. Does not validate the RFCOMM channel.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::Obex`] if the server rejects the connection or the response omits
    /// the `ConnectionId` header. Returns [`PbapError::Transport`] on I/O failure.
    pub async fn connect(stream: T) -> Result<Self, PbapError> {
        let mut transport = wrap(stream);
        let mut obex = ObexClient::new();
        let req = ObexClient::connect_request(&PBAP_UUID)?;
        transport.send(req).await?;
        let rsp = Self::recv(&mut transport).await?;
        obex.handle_connect_response(&rsp)?;
        Ok(Self { obex, transport })
    }

    /// `PullPhoneBook` for `path`. Device-reported order; silently skips unparseable vCards. Does not normalise numbers or filter `0.vcf`.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::ResponseTooLarge`] if the body exceeds 4 MiB.
    /// Returns [`PbapError::InvalidEncoding`] if the body is not valid UTF-8.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn pull_all(&mut self, path: PhonebookPath) -> Result<Vec<Contact>, PbapError> {
        let req = self.obex.get_request(
            b"x-bt/phonebook\x00",
            Some(path.pull_name()),
            Some(pull_all_params()),
        )?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        parse_contacts(&body)
    }

    /// Does not close the underlying stream; checks the response opcode only.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    pub async fn disconnect(mut self) -> Result<(), PbapError> {
        let req = self.obex.disconnect_request()?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(PbapError::ServerError(rsp.opcode.to_byte()));
        }
        Ok(())
    }

    /// `ListvCardObjects` for `path`. Device-reported order; does not filter `0.vcf` or fetch vCard content.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::CardListing`] if the listing XML cannot be parsed.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn list(&mut self, path: PhonebookPath) -> Result<Vec<CardEntry>, PbapError> {
        let req = self.obex.get_request(b"x-bt/vcard-listing\x00", Some(path.list_name()), None)?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        Ok(parse_card_listing(&body)?)
    }

    /// `PullvCardEntry` for the given handle. Silently ignores unrecognised vCard properties; does not normalise numbers.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::InvalidInput`] if `handle` is empty or contains CR or LF.
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::ResponseTooLarge`] if the body exceeds 4 MiB.
    /// Returns [`PbapError::InvalidEncoding`] if the body is not valid UTF-8.
    /// Returns [`PbapError::Contact`] if calcard cannot parse the vCard.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn pull(&mut self, path: PhonebookPath, handle: &str) -> Result<Contact, PbapError> {
        if handle.is_empty() || handle.contains(['\r', '\n']) {
            return Err(PbapError::InvalidInput(
                "handle must be non-empty and contain no CR or LF",
            ));
        }
        let name = path.entry_name(handle);
        let req =
            self.obex.get_request(b"x-bt/vcard\x00", Some(&name), Some(pull_entry_params()))?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        let text = std::str::from_utf8(&body).map_err(|_| PbapError::InvalidEncoding)?;
        Ok(Contact::from_vcard_str(text)?)
    }

    /// E.164-normalises both `number` and each contact's TEL before comparing. Returns the first match; skips `0.vcf`.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::InvalidInput`] if `number` contains CR or LF.
    /// Propagates all errors from [`list`](Self::list) and [`pull`](Self::pull).
    pub async fn find_by_number(
        &mut self,
        path: PhonebookPath,
        number: &str,
    ) -> Result<Option<Contact>, PbapError> {
        if number.contains(['\r', '\n']) {
            return Err(PbapError::InvalidInput("number must not contain CR or LF"));
        }
        let target = normalize_number(number);
        let entries = self.list(path).await?;
        for entry in &entries {
            if entry.handle() == "0.vcf" {
                continue;
            }
            let contact = self.pull(path, entry.handle()).await?;
            if contact.phones().iter().any(|p| normalize_number(p) == target) {
                return Ok(Some(contact));
            }
        }
        Ok(None)
    }

    async fn collect_body(&mut self) -> Result<Vec<u8>, PbapError> {
        let mut body = Vec::with_capacity(4096);
        loop {
            let rsp_bytes = Self::recv(&mut self.transport).await?;
            let rsp = ObexClient::parse_response(&rsp_bytes)?;
            if rsp.opcode.is_continue() {
                if let Some(chunk) = rsp.body_payload() {
                    let new_len =
                        body.len().checked_add(chunk.len()).ok_or(PbapError::ResponseTooLarge)?;
                    if new_len > MAX_BODY_BYTES {
                        return Err(PbapError::ResponseTooLarge);
                    }
                    body.extend_from_slice(chunk);
                }
                let cont = self.obex.get_continue_request()?;
                self.transport.send(cont).await?;
            } else if rsp.opcode.is_ok() {
                if let Some(chunk) = rsp.body_payload() {
                    let new_len =
                        body.len().checked_add(chunk.len()).ok_or(PbapError::ResponseTooLarge)?;
                    if new_len > MAX_BODY_BYTES {
                        return Err(PbapError::ResponseTooLarge);
                    }
                    body.extend_from_slice(chunk);
                }
                break;
            } else {
                return Err(PbapError::ServerError(rsp.opcode.to_byte()));
            }
        }
        Ok(body)
    }

    async fn recv(transport: &mut ObexTransport<T>) -> Result<Bytes, PbapError> {
        transport.next().await.ok_or(PbapError::UnexpectedEof)?.map_err(PbapError::Transport)
    }
}
