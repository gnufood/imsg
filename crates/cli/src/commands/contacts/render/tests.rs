//! Pure data transformation — no I/O, no fakes needed.
//!
//! `CardEntry` has no public constructor (`imsg-pbap` only builds one from a parsed wire
//! listing) so [`EntryView::from_entry`] can't be exercised here; its render-logic coverage
//! comes from directly-built [`EntryView`] values instead.

use std::fmt::Write as _;

use ipc::{PhoneDto, RefreshDto, SyncReportDto};
use pbap_core::{Contact, ContactError};
use session::contacts::{Refresh, SyncReport};
use store::{ContactEntryRow, ContactRow, PhoneField};

use super::*;

fn contact(name: Option<&str>, phones: &[&str]) -> Result<Contact, ContactError> {
    let mut vcard = String::from("BEGIN:VCARD\r\nVERSION:3.0\r\n");
    if let Some(n) = name {
        let _ = write!(vcard, "FN:{n}\r\n");
    }
    for p in phones {
        let _ = write!(vcard, "TEL:{p}\r\n");
    }
    vcard.push_str("END:VCARD\r\n");
    Contact::from_vcard_str(&vcard)
}

#[test]
fn render_contact_view_shows_unknown_for_missing_name() -> Result<(), ContactError> {
    let c = contact(None, &["+15550001"])?;
    let view = ContactView::from_contact(&c);
    assert_eq!(render_contact_view(&view, true), "(unknown)\n  +15550001");
    Ok(())
}

#[test]
fn render_contact_view_normalises_unless_raw() -> Result<(), ContactError> {
    // Numbers are normalised to E.164 at parse time (the PBAP ingress chokepoint); `raw` only
    // selects which retained form to render. UK trunk-prefix case: the country code (+44) is
    // enough context to resolve without a configured fallback region.
    let c = contact(Some("Ada Lovelace"), &["+44(0)1753866488"])?;
    let view = ContactView::from_contact(&c);
    assert_eq!(render_contact_view(&view, true), "Ada Lovelace\n  +44(0)1753866488");
    assert_eq!(render_contact_view(&view, false), "Ada Lovelace\n  +441753866488");
    Ok(())
}

#[test]
fn render_contact_view_falls_back_to_raw_when_unresolvable() -> Result<(), ContactError> {
    // No country code and no configured fallback region — MissingRegion, not a blank/error.
    let c = contact(Some("Bare Number"), &["4085550100"])?;
    let view = ContactView::from_contact(&c);
    assert_eq!(render_contact_view(&view, false), "Bare Number\n  4085550100");
    Ok(())
}

#[test]
fn render_contact_view_from_dto_matches_from_contact() {
    let dto = ContactDto {
        display_name: Some("Grace Hopper".to_owned()),
        uid: Some("uid-1".to_owned()),
        phones: vec![PhoneDto { raw: "+15550002".to_owned(), e164: None }],
    };
    let view = ContactView::from_dto(&dto);
    assert_eq!(render_contact_view(&view, true), "Grace Hopper\n  +15550002");
}

#[test]
fn render_contact_view_from_row_matches_from_contact() {
    let row = ContactRow {
        uid: "uid-2".to_owned(),
        display_name: Some("Margaret Hamilton".to_owned()),
        phones: vec![PhoneField::new("+15550003", None)],
    };
    let view = ContactView::from_row(&row);
    assert_eq!(render_contact_view(&view, true), "Margaret Hamilton\n  +15550003");
}

#[test]
fn render_contacts_view_joins_with_blank_line() -> Result<(), ContactError> {
    let a = contact(Some("Ada"), &["+1"])?;
    let b = contact(Some("Grace"), &["+2"])?;
    let views = vec![ContactView::from_contact(&a), ContactView::from_contact(&b)];
    assert_eq!(render_contacts_view(&views, true), "Ada\n  +1\n\nGrace\n  +2");
    Ok(())
}

#[test]
fn render_contacts_view_empty_is_empty_string() {
    assert_eq!(render_contacts_view(&[], true), "");
}

#[test]
fn render_entry_view_shows_key_and_name() {
    let view = EntryView { key: "1.vcf", name: Some("Ada Lovelace") };
    assert_eq!(render_entry_view(&view), "1.vcf  Ada Lovelace");
}

#[test]
fn render_entry_view_falls_back_to_key_alone() {
    let view = EntryView { key: "1.vcf", name: None };
    assert_eq!(render_entry_view(&view), "1.vcf");
}

#[test]
fn render_entry_view_from_dto_matches_key_and_name() {
    let dto = CardEntryDto { handle: "2.vcf".to_owned(), name: Some("Grace Hopper".to_owned()) };
    let view = EntryView::from_dto(&dto);
    assert_eq!(render_entry_view(&view), "2.vcf  Grace Hopper");
}

#[test]
fn render_entry_view_from_row_uses_uid_as_key() {
    let row = ContactEntryRow { uid: "uid-3".to_owned(), display_name: Some("Ada".to_owned()) };
    let view = EntryView::from_row(&row);
    assert_eq!(render_entry_view(&view), "uid-3  Ada");
}

#[test]
fn render_entries_view_joins_with_newline() {
    let views =
        vec![EntryView { key: "1.vcf", name: Some("Ada") }, EntryView { key: "2.vcf", name: None }];
    assert_eq!(render_entries_view(&views), "1.vcf  Ada\n2.vcf");
}

#[test]
fn render_entries_view_empty_is_empty_string() {
    assert_eq!(render_entries_view(&[]), "");
}

#[test]
fn offset_of_ignores_page_when_limit_is_none() {
    assert_eq!(offset_of(None, Some(3)), 0);
}

#[test]
fn offset_of_is_zero_when_limit_is_zero() {
    assert_eq!(offset_of(Some(0), Some(3)), 0);
}

#[test]
fn offset_of_defaults_page_to_one() {
    assert_eq!(offset_of(Some(20), None), 0);
    assert_eq!(offset_of(Some(20), Some(0)), 0);
    assert_eq!(offset_of(Some(20), Some(1)), 0);
}

#[test]
fn offset_of_computes_zero_indexed_offset() {
    assert_eq!(offset_of(Some(20), Some(2)), 20);
    assert_eq!(offset_of(Some(20), Some(3)), 40);
}

#[test]
fn offset_of_saturates_instead_of_overflowing() {
    assert_eq!(offset_of(Some(u16::MAX), Some(u16::MAX)), u16::MAX);
}

/// "0" is what made the old output misleading: an already-current cache must not render as a
/// count at all.
#[test]
fn render_sync_names_the_no_op_instead_of_counting_zero() {
    let dto = render_sync_dto(&SyncReportDto::UpToDate);
    let domain = render_sync_report(&SyncReport::UpToDate);

    assert_eq!(dto, "contacts already up to date");
    assert_eq!(dto, domain, "broker and spoke paths must word the same outcome identically");
}

#[test]
fn render_sync_reports_a_clean_refresh_as_a_plain_count() {
    let out = render_sync_dto(&SyncReportDto::Refreshed(RefreshDto {
        listed: 194,
        pull_failed: 0,
        no_uid: 0,
        written: 194,
        wiped: false,
    }));

    assert_eq!(out, "synced 194 contacts");
}

/// The point of the report: entries the device listed but the cache never got are named, not
/// silently folded into a smaller number.
#[test]
fn render_sync_names_discarded_entries() {
    let out = render_sync_dto(&SyncReportDto::Refreshed(RefreshDto {
        listed: 200,
        pull_failed: 4,
        no_uid: 2,
        written: 194,
        wiped: false,
    }));

    assert_eq!(out, "synced 194 contacts (4 failed to fetch, 2 without a UID)");
}

#[test]
fn render_sync_flags_a_rebuilt_cache() {
    let out = render_sync_report(&SyncReport::Refreshed(Refresh {
        listed: 3,
        pull_failed: 0,
        no_uid: 0,
        written: 3,
        wiped: true,
    }));

    assert_eq!(out, "synced 3 contacts (cache rebuilt)");
}

/// Singulars read as English, not `1 contacts`/`1 failed`.
#[test]
fn render_sync_singularises() {
    let out = render_sync_dto(&SyncReportDto::Refreshed(RefreshDto {
        listed: 3,
        pull_failed: 1,
        no_uid: 1,
        written: 1,
        wiped: false,
    }));

    assert_eq!(out, "synced 1 contact (1 failed to fetch, 1 without a UID)");
}
