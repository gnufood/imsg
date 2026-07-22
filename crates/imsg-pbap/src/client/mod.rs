//! PBAP client state machine — session setup and phonebook pull requests.

mod io;

use formats::vcard::Contact;
use futures::SinkExt;
use obex_core::client::ObexClient;
use obex_core::{wrap, ObexTransport};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    contacts::{parse_card_listing, parse_contacts, CardEntry},
    metadata::PhonebookMetadata,
    params::{
        connect_params, list_params, metadata_params, pull_all_params, pull_entry_params,
        search_params,
    },
    phonebook::{PhonebookPath, SearchAttribute},
    PbapError,
};

const PBAP_UUID: [u8; 16] = [
    0x79, 0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

/// Owns the OBEX state machine and framed I/O. Obtain via [`connect`](Self::connect).
pub struct PbapClient<T> {
    obex: ObexClient,
    transport: ObexTransport<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> PbapClient<T> {
    /// Sends PBAP PSE UUID as OBEX `Target` plus `PBAPSupportedFeatures` (`Download` |
    /// `DatabaseIdentifier` | `FolderVersionCounters`) and validates the server response. Does
    /// not validate the RFCOMM channel.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::Obex`] if the server rejects the connection or the response omits
    /// the `ConnectionId` header. Returns [`PbapError::Transport`] on I/O failure.
    pub async fn connect(stream: T) -> Result<Self, PbapError> {
        let mut transport = wrap(stream);
        let mut obex = ObexClient::new();
        let req = ObexClient::connect_request(&PBAP_UUID, Some(connect_params()))?;
        transport.send(req).await?;
        let rsp = Self::recv(&mut transport).await?;
        obex.handle_connect_response(&rsp)?;
        Ok(Self { obex, transport })
    }

    /// `PullPhoneBook` for `path`, windowed to `limit` entries starting at `offset` (device-side
    /// `MaxListCount`/`ListStartOffset`); `limit: None` fetches everything. Device-reported
    /// order; silently skips unparseable vCards. Does not normalise numbers or filter `0.vcf`.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::ResponseTooLarge`] if the body exceeds 4 MiB.
    /// Returns [`PbapError::InvalidEncoding`] if the body is not valid UTF-8.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn pull_all(
        &mut self,
        path: PhonebookPath,
        limit: Option<u16>,
        offset: u16,
    ) -> Result<Vec<Contact>, PbapError> {
        let req = self.obex.get_request(
            b"x-bt/phonebook\x00",
            Some(path.pull_name()),
            Some(pull_all_params(limit, offset)),
        )?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        parse_contacts(&body)
    }

    /// Metadata-only `PullPhoneBook` for `path` (`MaxListCount=0`): no vCard body is fetched, just
    /// whatever `PhonebookSize`/`DatabaseIdentifier`/version-counter fields the device includes in
    /// the response's `AppParams`. Fields the device omits come back `None` in
    /// [`PhonebookMetadata`].
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::ResponseTooLarge`] if the body exceeds 4 MiB.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn phonebook_metadata(
        &mut self,
        path: PhonebookPath,
    ) -> Result<PhonebookMetadata, PbapError> {
        let req = self.obex.get_request(
            b"x-bt/phonebook\x00",
            Some(path.pull_name()),
            Some(metadata_params()),
        )?;
        self.transport.send(req).await?;
        let (_, app_params) = self.collect_response().await?;
        Ok(PhonebookMetadata::parse(app_params.as_deref().unwrap_or_default()))
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

    /// `ListvCardObjects` for `path`, windowed to `limit` entries starting at `offset`
    /// (device-side `MaxListCount`/`ListStartOffset`); `limit: None` and `offset: 0` omits both
    /// and fetches everything. Device-reported order; does not filter `0.vcf` or fetch vCard
    /// content.
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::CardListing`] if the listing XML cannot be parsed.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn list(
        &mut self,
        path: PhonebookPath,
        limit: Option<u16>,
        offset: u16,
    ) -> Result<Vec<CardEntry>, PbapError> {
        let req = self.obex.get_request(
            b"x-bt/vcard-listing\x00",
            Some(path.list_name()),
            list_params(limit, offset),
        )?;
        self.transport.send(req).await?;
        let body = self.collect_body().await?;
        Ok(parse_card_listing(&body)?)
    }

    /// `ListvCardObjects` filtered device-side by `SearchAttribute`/`SearchValue`: entries whose
    /// `attribute` field matches `value`. Windowed the same way as [`list`](Self::list).
    ///
    /// # Errors
    ///
    /// Returns [`PbapError::InvalidInput`] if `value` contains CR or LF or exceeds 255 UTF-8
    /// bytes. Returns [`PbapError::ServerError`] if the remote returns a non-OK response.
    /// Returns [`PbapError::CardListing`] if the listing XML cannot be parsed.
    /// Returns [`PbapError::Transport`] or [`PbapError::Obex`] on lower-layer failure.
    pub async fn search(
        &mut self,
        path: PhonebookPath,
        attribute: SearchAttribute,
        value: &str,
        limit: Option<u16>,
        offset: u16,
    ) -> Result<Vec<CardEntry>, PbapError> {
        if value.contains(['\r', '\n']) {
            return Err(PbapError::InvalidInput("search value must not contain CR or LF"));
        }
        if value.len() > 255 {
            return Err(PbapError::InvalidInput("search value exceeds 255 UTF-8 bytes"));
        }
        let req = self.obex.get_request(
            b"x-bt/vcard-listing\x00",
            Some(path.list_name()),
            Some(search_params(attribute, value, limit, offset)),
        )?;
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
}
