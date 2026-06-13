//! `contacts` subcommand: pull, list, fetch, or reverse-lookup PBAP contacts.

use std::fmt::Write as _;

use anyhow::Result;
use config::Config;
use pbap_core::{normalize_number, Contact};

use crate::cli::{path_of, PathArg};
use transport::iroh::Endpoint;

use crate::commands::conn;

/// Returns the sub-slice of `items` for the requested page.
///
/// `limit = None` returns the full slice. `limit = Some(0)` returns an empty slice.
/// `page` is 1-indexed; `None` or `0` is treated as page 1. Offset is clamped to
/// `items.len()` — never panics on out-of-range page numbers.
fn paginate<T>(items: &[T], limit: Option<u16>, page: Option<u16>) -> &[T] {
    let n = match limit {
        None => return items,
        Some(0) => return &[],
        Some(n) => usize::from(n),
    };
    let p = usize::from(page.unwrap_or(1).max(1));
    let start = p.saturating_sub(1).saturating_mul(n).min(items.len());
    let end = start.saturating_add(n).min(items.len());
    items.get(start..end).unwrap_or_default()
}

/// Renders one contact: name (`(unknown)` when the vCard carries no FN), then each phone
/// number on its own indented line. Normalises numbers to E.164 unless `raw` is true.
fn render_contact(c: &Contact, raw: bool) -> String {
    let name = c.display_name.as_deref().unwrap_or("(unknown)");
    let mut out =
        String::with_capacity(name.len().saturating_add(c.phones().len().saturating_mul(20)));
    out.push_str(name);
    for tel in c.phones() {
        let number = if raw { tel.to_owned() } else { normalize_number(tel) };
        let _ = write!(out, "\n  {number}");
    }
    out
}

/// Renders contacts as [`render_contact`] blocks separated by a blank line.
/// An empty slice renders as the empty string.
fn render_contacts(contacts: &[Contact], raw: bool) -> String {
    contacts.iter().map(|c| render_contact(c, raw)).collect::<Vec<_>>().join("\n\n")
}

/// Renders one vCard-listing line: the handle followed by its name, or the handle alone
/// when the listing entry carries no name.
fn render_entry(handle: &str, name: Option<&str>) -> String {
    name.map_or_else(|| handle.to_owned(), |n| format!("{handle}  {n}"))
}

/// Operation parameters for the `contacts` subcommand.
pub(crate) struct ContactsOpts {
    /// List handles and names only, without fetching full vCards.
    pub list: bool,
    /// Fetch a single contact by vCard handle.
    pub get: Option<String>,
    /// Reverse-lookup a contact by phone number.
    pub lookup: Option<String>,
    /// Phonebook path to query.
    pub path: PathArg,
    /// When true, skip E.164 normalisation and emit numbers as stored. No effect on `--list`.
    pub raw: bool,
    /// Maximum contacts per page; `None` shows all.
    pub limit: Option<u16>,
    /// 1-indexed page number; ignored when `limit` is absent.
    pub page: Option<u16>,
}

/// Connects to PBAP, runs exactly one of reverse-lookup / fetch-one / list / full-pull
/// (clap enforces the flags are mutually exclusive), and returns the result rendered.
///
/// Normalises phone numbers to E.164 in rendered output unless `raw` is true. `raw` has
/// no effect on `--list` (handle/name display only). `limit` and `page` apply to pull-all
/// and list modes; single-result modes (`--get`, `--lookup`) ignore them. `page` is
/// 1-indexed and ignored when `limit` is absent.
///
/// The OBEX disconnect is best-effort: a teardown failure is logged as a warning and never
/// discards the result, which is always returned to the caller for display.
///
/// # Errors
///
/// Returns an error if the connection or the chosen pull/list/lookup fails. A failed
/// disconnect is warned, not propagated.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: ContactsOpts,
) -> Result<String> {
    let ContactsOpts { list, get, lookup, path, raw, limit, page } = opts;
    let pb = path_of(path);
    let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
    let out = if let Some(number) = lookup {
        client
            .find_by_number(pb, &number)
            .await?
            .map_or_else(|| format!("no contact found for {number}"), |c| render_contact(&c, raw))
    } else if let Some(handle) = get {
        render_contact(&client.pull(pb, &handle).await?, raw)
    } else if list {
        let entries = client.list(pb).await?;
        paginate(&entries, limit, page)
            .iter()
            .map(|e| render_entry(e.handle(), e.name()))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        let contacts = client.pull_all(pb).await?;
        let page_contacts = paginate(&contacts, limit, page);
        if page_contacts.is_empty() {
            "(no contacts)".to_owned()
        } else {
            render_contacts(page_contacts, raw)
        }
    };
    if let Err(e) = client.disconnect().await {
        tracing::warn!("PBAP disconnect failed: {e}");
    }
    Ok(out)
}
