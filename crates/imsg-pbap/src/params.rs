//! PBAP application parameter encoder functions.

use bytes::Bytes;

/// `PullPhoneBook` app-params: `Format=vcard30`, `MaxListCount=65535`.
///
/// Wire encoding: `[0x07, 0x01, 0x01, 0x04, 0x02, 0xFF, 0xFF]`.
#[must_use]
pub const fn pull_all_params() -> Bytes {
    Bytes::from_static(b"\x07\x01\x01\x04\x02\xff\xff")
}

/// `PullvCardEntry` app-params: `Format=vcard30` only.
///
/// Wire encoding: `[0x07, 0x01, 0x01]`.
#[must_use]
pub const fn pull_entry_params() -> Bytes {
    Bytes::from_static(b"\x07\x01\x01")
}
