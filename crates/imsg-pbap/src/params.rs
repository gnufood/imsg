//! PBAP application parameter encoder functions.

use bytes::Bytes;

use crate::phonebook::SearchAttribute;

/// CONNECT-time `PBAPSupportedFeatures` app-params: `Download | DatabaseIdentifier |
/// FolderVersionCounters` (`0x0000000D`).
///
/// Wire encoding: `[0x10, 0x04, 0x00, 0x00, 0x00, 0x0D]`. Declaring exactly this
/// device-confirmed bit combination at CONNECT is required to unlock `DatabaseIdentifier`/
/// `PrimaryVersionCounter`/`SecondaryVersionCounter` in later `PullPhoneBook` responses.
#[must_use]
pub const fn connect_params() -> Bytes {
    Bytes::from_static(b"\x10\x04\x00\x00\x00\x0d")
}

/// `PropertySelector`: `VERSION | FN | TEL | UID` (`0x0000000000200083`) — every vCard field this
/// crate actually parses. Trims a PHOTO-laden pull from ~400KB to ~18KB (device-confirmed on
/// `PullPhoneBook`; not separately confirmed on `PullvCardEntry`, but it's the same standard
/// `AppParams` tag on the same GET-style operation).
///
/// Wire encoding: `[0x06, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x83]`.
const PROPERTY_SELECTOR: &[u8] = b"\x06\x08\x00\x00\x00\x00\x00\x20\x00\x83";

/// `PullPhoneBook` app-params: `Format=vcard30`, [`PROPERTY_SELECTOR`], `MaxListCount=limit`.
///
/// `0xFFFF` when `limit` is `None`, matching the "fetch everything" default. `ListStartOffset` is
/// included only when `offset != 0`, so the wire bytes for a full unwindowed pull are unchanged
/// from before pagination support existed.
#[must_use]
pub fn pull_all_params(limit: Option<u16>, offset: u16) -> Bytes {
    let mut out = Vec::with_capacity(21);
    out.extend_from_slice(b"\x07\x01\x01");
    out.extend_from_slice(PROPERTY_SELECTOR);
    out.extend_from_slice(&[0x04, 0x02]);
    out.extend_from_slice(&limit.unwrap_or(u16::MAX).to_be_bytes());
    if offset != 0 {
        out.extend_from_slice(&[0x05, 0x02]);
        out.extend_from_slice(&offset.to_be_bytes());
    }
    Bytes::from(out)
}

/// `ListvCardObjects` app-params: `MaxListCount`/`ListStartOffset`.
///
/// Included only when `limit` is `Some` or `offset != 0`. Returns `None` for the "fetch
/// everything" default, so the wire bytes are unchanged from before pagination support existed
/// (no `AppParams` header at all).
#[must_use]
pub fn list_params(limit: Option<u16>, offset: u16) -> Option<Bytes> {
    if limit.is_none() && offset == 0 {
        return None;
    }
    let mut out = Vec::with_capacity(8);
    if let Some(max_count) = limit {
        out.extend_from_slice(&[0x04, 0x02]);
        out.extend_from_slice(&max_count.to_be_bytes());
    }
    if offset != 0 {
        out.extend_from_slice(&[0x05, 0x02]);
        out.extend_from_slice(&offset.to_be_bytes());
    }
    Some(Bytes::from(out))
}

/// `ListvCardObjects` search app-params: `SearchAttribute`, `SearchValue` (raw UTF-8, length-
/// prefixed — not null-terminated), plus any windowing from [`list_params`].
///
/// `value` must already be validated by the caller: at most 255 UTF-8 bytes (the TLV length
/// field is one byte).
#[must_use]
pub fn search_params(
    attribute: SearchAttribute,
    value: &str,
    limit: Option<u16>,
    offset: u16,
) -> Bytes {
    let mut out = Vec::with_capacity(5_usize.saturating_add(value.len()));
    out.extend_from_slice(&[0x03, 0x01, attribute.to_wire()]);
    out.push(0x02);
    out.push(u8::try_from(value.len()).unwrap_or(u8::MAX));
    out.extend_from_slice(value.as_bytes());
    if let Some(window) = list_params(limit, offset) {
        out.extend_from_slice(&window);
    }
    Bytes::from(out)
}

/// `PullvCardEntry` app-params: `Format=vcard30`, [`PROPERTY_SELECTOR`].
///
/// Wire encoding: `[0x07, 0x01, 0x01, 0x06, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00,
/// 0x83]`.
#[must_use]
pub fn pull_entry_params() -> Bytes {
    let mut out = Vec::with_capacity(13);
    out.extend_from_slice(b"\x07\x01\x01");
    out.extend_from_slice(PROPERTY_SELECTOR);
    Bytes::from(out)
}

/// `PullPhoneBook` metadata-only app-params: `Format=vcard30`, `MaxListCount=0`.
///
/// Triggers a response with no vCard body — just `PhonebookSize`/`DatabaseIdentifier`/
/// version-counter `AppParams` fields (see
/// [`PhonebookMetadata`](crate::metadata::PhonebookMetadata)).
///
/// Wire encoding: `[0x07, 0x01, 0x01, 0x04, 0x02, 0x00, 0x00]`.
#[must_use]
pub const fn metadata_params() -> Bytes {
    Bytes::from_static(b"\x07\x01\x01\x04\x02\x00\x00")
}
