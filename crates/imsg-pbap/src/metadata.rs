//! Phonebook metadata parsed from a `PullPhoneBook` response's `AppParams` header.

/// Metadata-only `PullPhoneBook` response fields, requested via `MaxListCount=0`.
///
/// Fields are `None` when the device omitted them — version counters and `database_id` require
/// declaring `DatabaseIdentifier`/`FolderVersionCounters` in `PBAPSupportedFeatures` at CONNECT
/// time (see [`PbapClient::connect`](crate::client::PbapClient::connect)).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhonebookMetadata {
    /// Total contact count in the requested phonebook.
    pub size: Option<u16>,
    /// Fixed identifier for the phone's contacts database instance. Changes only if the
    /// underlying database itself changes (e.g. restored from a different backup); stable across
    /// reconnects otherwise.
    pub database_id: Option<[u8; 16]>,
    /// Monotonic counter that increments when the phonebook's handle set changes (contacts
    /// added or removed).
    pub primary_version: Option<[u8; 16]>,
    /// Monotonic counter that increments when an existing entry's properties change.
    pub secondary_version: Option<[u8; 16]>,
}

impl PhonebookMetadata {
    /// Parses recognised tags out of a raw `AppParams` TLV byte sequence. Unrecognised tags and
    /// truncated trailing TLVs are silently ignored — an absent/malformed field just stays
    /// `None`, matching how the rest of PBAP already treats optional device-reported data.
    pub(crate) fn parse(app_params: &[u8]) -> Self {
        let mut out = Self::default();
        for (tag, value) in tlv_entries(app_params) {
            match tag {
                0x08 => {
                    if let Ok(bytes) = value.try_into() {
                        out.size = Some(u16::from_be_bytes(bytes));
                    }
                }
                0x0A => out.primary_version = value.try_into().ok(),
                0x0B => out.secondary_version = value.try_into().ok(),
                0x0D => out.database_id = value.try_into().ok(),
                _ => {}
            }
        }
        out
    }
}

// iterates (tag, value) TLV entries in an OBEX AppParams byte sequence
fn tlv_entries(data: &[u8]) -> impl Iterator<Item = (u8, &[u8])> {
    let mut rest = data;
    std::iter::from_fn(move || {
        let (&tag, after_tag) = rest.split_first()?;
        let (&len, after_len) = after_tag.split_first()?;
        let value = after_len.get(..usize::from(len))?;
        rest = after_len.get(usize::from(len)..)?;
        Some((tag, value))
    })
}

#[cfg(test)]
mod tests;
