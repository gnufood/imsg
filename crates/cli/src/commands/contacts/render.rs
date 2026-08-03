//! Presentation for contacts — turns any of the three backing shapes (live `pbap_core` types,
//! broker `ipc` DTOs, cached `store` rows) into display strings. No I/O.

use std::fmt::Write as _;

use ipc::{CardEntryDto, ContactDto, SyncReportDto};
use pbap_core::{CardEntry, Contact};
use session::contacts::SyncReport;
use store::{ContactEntryRow, ContactRow, PhoneField};

/// One phone number borrowed from its source, exposing both forms so the renderer can pick.
///
/// `display` is the E.164 canonical form when the number resolved, else the raw form; numbers are
/// normalised at ingress, so this borrow never re-parses.
struct PhoneRef<'a> {
    raw: &'a str,
    display: &'a str,
}

impl<'a> From<&'a PhoneField> for PhoneRef<'a> {
    fn from(p: &'a PhoneField) -> Self {
        Self { raw: p.raw(), display: p.display() }
    }
}

impl<'a> From<&'a ipc::PhoneDto> for PhoneRef<'a> {
    fn from(p: &'a ipc::PhoneDto) -> Self {
        Self { raw: &p.raw, display: p.e164.as_deref().unwrap_or(&p.raw) }
    }
}

/// A full contact, borrowed from whichever of the three sources produced it.
pub(super) struct ContactView<'a> {
    name: Option<&'a str>,
    phones: Vec<PhoneRef<'a>>,
}

impl<'a> ContactView<'a> {
    pub(super) fn from_contact(c: &'a Contact) -> Self {
        Self {
            name: c.display_name.as_deref(),
            phones: c.phones().iter().map(Into::into).collect(),
        }
    }

    pub(super) fn from_dto(c: &'a ContactDto) -> Self {
        Self { name: c.display_name.as_deref(), phones: c.phones.iter().map(Into::into).collect() }
    }

    pub(super) fn from_row(c: &'a ContactRow) -> Self {
        Self { name: c.display_name.as_deref(), phones: c.phones.iter().map(Into::into).collect() }
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

/// Renders one contact: name (`(unknown)` when absent), then each phone number on its own
/// indented line. Numbers are normalised to E.164 at ingress; `raw` selects the original
/// device-reported form instead.
pub(super) fn render_contact_view(v: &ContactView, raw: bool) -> String {
    let name = v.name.unwrap_or("(unknown)");
    let mut out =
        String::with_capacity(name.len().saturating_add(v.phones.len().saturating_mul(20)));
    out.push_str(name);
    for tel in &v.phones {
        let number = if raw { tel.raw } else { tel.display };
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

/// Renders a broker-reported sync outcome. Paired with [`render_sync_report`], which words the
/// spoke path's domain-typed outcome identically — same split as [`ContactView`]'s per-source
/// constructors, since the two paths never share a type.
pub(super) fn render_sync_dto(r: &SyncReportDto) -> String {
    match *r {
        SyncReportDto::UpToDate => up_to_date(),
        SyncReportDto::Refreshed(f) => render_refresh(f.pull_failed, f.no_uid, f.written, f.wiped),
    }
}

/// Renders a directly-synced (spoke path) outcome; see [`render_sync_dto`].
pub(super) fn render_sync_report(r: &SyncReport) -> String {
    match *r {
        SyncReport::UpToDate => up_to_date(),
        SyncReport::Refreshed(f) => render_refresh(f.pull_failed, f.no_uid, f.written, f.wiped),
    }
}

fn up_to_date() -> String {
    "contacts already up to date".to_owned()
}

/// `written` leads, since that's what the cache actually holds; anything lost or invalidated
/// follows in parentheses so a partial sync can't read as a clean one.
fn render_refresh(pull_failed: usize, no_uid: usize, written: usize, wiped: bool) -> String {
    let mut out = format!("synced {written} {}", plural(written, "contact"));
    let mut notes: Vec<String> = Vec::new();
    if wiped {
        notes.push("cache rebuilt".to_owned());
    }
    if pull_failed > 0 {
        notes.push(format!("{pull_failed} failed to fetch"));
    }
    if no_uid > 0 {
        notes.push(format!("{no_uid} without a UID"));
    }
    if !notes.is_empty() {
        let _ = write!(out, " ({})", notes.join(", "));
    }
    out
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_owned()
    } else {
        format!("{word}s")
    }
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
