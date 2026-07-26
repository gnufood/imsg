//! Presentation for contacts — turns any of the three backing shapes (live `pbap_core` types,
//! broker `ipc` DTOs, cached `store` rows) into display strings. No I/O.

use std::fmt::Write as _;

use formats::phone::{normalize_number, Normalization};
use ipc::{CardEntryDto, ContactDto};
use pbap_core::{CardEntry, Contact};
use store::{ContactEntryRow, ContactRow};

/// A full contact, borrowed from whichever of the three sources produced it.
pub(super) struct ContactView<'a> {
    name: Option<&'a str>,
    phones: &'a [String],
}

impl<'a> ContactView<'a> {
    pub(super) fn from_contact(c: &'a Contact) -> Self {
        Self { name: c.display_name.as_deref(), phones: c.phones() }
    }

    pub(super) fn from_dto(c: &'a ContactDto) -> Self {
        Self { name: c.display_name.as_deref(), phones: &c.phones }
    }

    pub(super) fn from_row(c: &'a ContactRow) -> Self {
        Self { name: c.display_name.as_deref(), phones: &c.phones }
    }
}

/// A phonebook listing entry, borrowed from whichever of the three sources produced it.
///
/// `key` is a PBAP handle live/via broker, or a cached UID once opted in — whatever `--list`
/// prints in that mode is what `--get` accepts back.
pub(super) struct EntryView<'a> {
    key: &'a str,
    name: Option<&'a str>,
}

impl<'a> EntryView<'a> {
    pub(super) fn from_entry(e: &'a CardEntry) -> Self {
        Self { key: e.handle(), name: e.name() }
    }

    pub(super) fn from_dto(e: &'a CardEntryDto) -> Self {
        Self { key: &e.handle, name: e.name.as_deref() }
    }

    pub(super) fn from_row(e: &'a ContactEntryRow) -> Self {
        Self { key: &e.uid, name: e.display_name.as_deref() }
    }
}

/// Resolves one TEL value for display: the E.164 form when it normalises cleanly, the raw
/// value otherwise — normalisation is best-effort and never blanks out a number.
fn display_number(tel: &str, raw: bool) -> String {
    if raw {
        return tel.to_owned();
    }
    match normalize_number(tel, None) {
        Normalization::E164(number) => number,
        _ => tel.to_owned(),
    }
}

/// Renders one contact: name (`(unknown)` when absent), then each phone number on its own
/// indented line. Normalises numbers to E.164 unless `raw` is true.
pub(super) fn render_contact_view(v: &ContactView, raw: bool) -> String {
    let name = v.name.unwrap_or("(unknown)");
    let mut out =
        String::with_capacity(name.len().saturating_add(v.phones.len().saturating_mul(20)));
    out.push_str(name);
    for tel in v.phones {
        let number = display_number(tel, raw);
        let _ = write!(out, "\n  {number}");
    }
    out
}

/// Renders contacts as [`render_contact_view`] blocks separated by a blank line. An empty slice
/// renders as the empty string.
pub(super) fn render_contacts_view(views: &[ContactView], raw: bool) -> String {
    views.iter().map(|v| render_contact_view(v, raw)).collect::<Vec<_>>().join("\n\n")
}

/// Renders one listing entry: the key followed by its name, or the key alone when unnamed.
pub(super) fn render_entry_view(v: &EntryView) -> String {
    v.name.map_or_else(|| v.key.to_owned(), |n| format!("{}  {n}", v.key))
}

/// Renders listing entries one per line. An empty slice renders as the empty string.
pub(super) fn render_entries_view(views: &[EntryView]) -> String {
    views.iter().map(render_entry_view).collect::<Vec<_>>().join("\n")
}

/// Converts a 1-indexed page number to a device/broker-side row offset.
///
/// `limit = None` or `Some(0)` means pagination doesn't apply, so the offset is always `0`.
/// `page = None` or `Some(0)` is treated as page 1. Saturates instead of overflowing `u16`.
pub(super) fn offset_of(limit: Option<u16>, page: Option<u16>) -> u16 {
    let Some(n) = limit else { return 0 };
    if n == 0 {
        return 0;
    }
    let p = page.unwrap_or(1).max(1);
    p.saturating_sub(1).saturating_mul(n)
}

#[cfg(test)]
mod tests;
