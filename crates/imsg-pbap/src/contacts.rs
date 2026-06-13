//! Contact and vCard entry retrieval from the remote PBAP server.

use formats::vcard::Contact;
use quick_xml::{events::Event, Reader};
use thiserror::Error;

use crate::PbapError;

/// PBAP vCard-listing XML parsing errors — quick-xml parse, attribute decode, UTF-8 validation, and missing handle attribute.
#[derive(Debug, Error)]
pub enum CardListingError {
    /// Malformed XML in the vCard listing.
    #[error("XML error: {0}")]
    Parse(#[from] quick_xml::Error),
    /// Invalid or malformed attribute encoding.
    #[error("attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),
    /// Handle or name attribute bytes are not valid UTF-8.
    #[error("non-UTF-8 attribute value")]
    Utf8(#[from] std::string::FromUtf8Error),
    /// A `<card>` element is present but the required `handle` attribute is absent.
    #[error("card element missing required handle attribute")]
    MissingHandle,
}

/// A phonebook entry from a `ListvCardObjects` response.
///
/// The handle is the opaque identifier to pass to
/// [`PbapClient::pull`](crate::client::PbapClient::pull).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardEntry {
    handle: String,
    name: Option<String>,
}

impl CardEntry {
    pub(crate) const fn new(handle: String, name: Option<String>) -> Self {
        Self { handle, name }
    }

    /// Opaque vCard handle assigned by the remote, e.g. `"41.vcf"`. Non-empty.
    ///
    /// Pass directly to [`PbapClient::pull`](crate::client::PbapClient::pull).
    #[must_use]
    pub fn handle(&self) -> &str {
        &self.handle
    }

    /// Display name from the listing XML `name` attribute; `None` if the attribute was absent.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

// document order; errors on <card> missing handle; ignores non-card elements
pub(crate) fn parse_card_listing(xml: &[u8]) -> Result<Vec<CardEntry>, CardListingError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut entries = Vec::new();
    let mut buf = Vec::with_capacity(64);
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"card" => {
                let mut handle: Option<String> = None;
                let mut name: Option<String> = None;
                for attr in e.attributes() {
                    let a = attr?;
                    match a.key.as_ref() {
                        b"handle" => handle = Some(String::from_utf8(a.value.into_owned())?),
                        b"name" => name = Some(String::from_utf8(a.value.into_owned())?),
                        _ => {}
                    }
                }
                entries.push(CardEntry::new(handle.ok_or(CardListingError::MissingHandle)?, name));
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(entries)
}

/// Strips all non-digit, non-`+` characters. Prepends `+` for 11-digit strings starting with
/// `1`; prepends `+1` for 10-digit strings. No E.164 validation.
#[must_use]
pub fn normalize_number(s: &str) -> String {
    let mut d = String::with_capacity(s.len());
    d.extend(s.chars().filter(|c| c.is_ascii_digit() || *c == '+'));
    if d.starts_with('1') && d.len() == 11 {
        d.insert(0, '+');
    } else if d.len() == 10 {
        d.insert_str(0, "+1");
    }
    d
}

// silently skips cards calcard cannot parse; errors on non-UTF-8 blob
pub(crate) fn parse_contacts(body: &[u8]) -> Result<Vec<Contact>, PbapError> {
    let text = std::str::from_utf8(body).map_err(|_| PbapError::InvalidEncoding)?;
    Ok(text
        .split_inclusive("END:VCARD\r\n")
        .filter(|block| block.contains("BEGIN:VCARD"))
        .filter_map(|block| Contact::from_vcard_str(block).ok())
        .collect())
}
