//! Cached PBAP contact reads — direct `Store` queries, no network at all.
//!
//! `--get`'s key is the cached UID here, not a PBAP handle — the store never caches those (see
//! `render.rs`'s `EntryView` doc) — matching whatever `--list` prints in this mode.

use anyhow::Result;
use store::Store;

use super::render::{
    offset_of, render_contact_view, render_contacts_view, render_entries_view, ContactView,
    EntryView,
};
use super::ContactsOpts;

/// Reads cached phonebook data from the local store, with a freshness footer. Never opens a
/// Bluetooth connection. Exactly one of `--lookup`/`--get`/`--list`/pull-all (the default) runs,
/// per `opts`.
///
/// # Errors
///
/// Returns an error if the store read or `contacts_synced_at` query fails, or (for `--get`) if
/// `uid` isn't cached.
pub(crate) async fn run(opts: &ContactsOpts, store: &Store) -> Result<String> {
    let mut out = render(opts, store).await?;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&crate::commands::freshness_line(store.contacts_synced_at().await?));
    Ok(out)
}

async fn render(opts: &ContactsOpts, store: &Store) -> Result<String> {
    if let Some(number) = &opts.lookup {
        // Already normalised by the dispatcher (crate::commands::dispatch::run_contacts), before
        // the store/live fork.
        return Ok(store.lookup_contact(number).await?.map_or_else(
            || format!("no contact found for {number}"),
            |c| render_contact_view(&ContactView::from_row(&c), opts.raw),
        ));
    }
    if let Some(uid) = &opts.get {
        let row = store
            .get_contact(uid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("contact {uid} not found in local store"))?;
        return Ok(render_contact_view(&ContactView::from_row(&row), opts.raw));
    }
    let offset = offset_of(opts.limit, opts.page);
    let limit = opts.limit.unwrap_or(u16::MAX);
    if opts.list {
        let entries = store.list_contacts(limit, offset).await?;
        let views: Vec<EntryView> = entries.iter().map(EntryView::from_row).collect();
        return Ok(render_entries_view(&views));
    }
    let contacts = store.all_contacts(limit, offset).await?;
    if contacts.is_empty() {
        return Ok("(no contacts)".to_owned());
    }
    let views: Vec<ContactView> = contacts.iter().map(ContactView::from_row).collect();
    Ok(render_contacts_view(&views, opts.raw))
}
