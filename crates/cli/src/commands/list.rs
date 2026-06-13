//! `list` subcommand: navigate to a folder, list its messages, render a table.

use std::fmt::Write as _;

use anyhow::Result;
use config::Config;
use map_core::client::MapClient;
use map_core::folders::Folder;
use map_core::messages::{ListMessagesFilter, MessageEntry, ReadStatus};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::cli::{folder_of, FolderArg};
use transport::iroh::Endpoint;

use crate::commands::conn;
use crate::fmt::fmt_datetime;

/// Builds a [`ListMessagesFilter`] from `list` flags.
///
/// Unset `limit`/`offset` fall back to the filter defaults; `unread` restricts to unread
/// messages. `from`/`since` pass through unvalidated — the params layer rejects malformed
/// values (e.g. embedded null bytes) at request-encoding time.
pub(crate) fn build_filter(
    unread: bool,
    from: Option<String>,
    since: Option<String>,
    limit: Option<u16>,
    offset: Option<u16>,
) -> ListMessagesFilter {
    let defaults = ListMessagesFilter::default();
    ListMessagesFilter {
        max_count: limit.unwrap_or(defaults.max_count),
        offset: offset.unwrap_or(defaults.offset),
        read_status: unread.then_some(ReadStatus::Unread),
        originating_address: from,
        period_begin: since,
        period_end: None,
    }
}

/// Returns entries from `entries` that satisfy all supplied filter criteria.
///
/// `from` matches `sender_addressing` for received messages and `recipient_addressing`
/// for sent. `since`/`until` compare the first 15 chars of `datetime` lexicographically
/// (MAP datetime `YYYYMMDDTHHMMSS` sorts correctly by prefix; timezone suffixes are ignored).
/// Criteria that are `None` are not applied. Does not validate datetime format or normalise
/// phone numbers; does not error when no entries match.
fn apply_filters<'a>(
    entries: &'a [MessageEntry],
    from: Option<&str>,
    since: Option<&str>,
    until: Option<&str>,
) -> Vec<&'a MessageEntry> {
    entries
        .iter()
        .filter(|e| {
            if let Some(addr) = from {
                let peer = if e.sent { &e.recipient_addressing } else { &e.sender_addressing };
                if peer != addr {
                    return false;
                }
            }
            if let Some(t) = since {
                if dt_prefix(&e.datetime) < dt_prefix(t) {
                    return false;
                }
            }
            if let Some(t) = until {
                if dt_prefix(&e.datetime) > dt_prefix(t) {
                    return false;
                }
            }
            true
        })
        .collect()
}

// Truncates to at most 15 bytes at a valid char boundary — phone datetime is expected
// ASCII but OBEX responses are untrusted; a non-ASCII char straddling byte 15 must not panic.
fn dt_prefix(s: &str) -> &str {
    let n = s.len().min(15);
    let i = (0..=n).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
    s.get(..i).unwrap_or("")
}

/// Renders message entries as one line each: unread marker, formatted datetime, sender name
/// or number, and subject. When `long` is true, the raw MAP handle is prepended so it
/// can be passed to `get` or `delete`. Returns a placeholder when `entries` is empty.
pub(crate) fn render(entries: &[&MessageEntry], long: bool) -> String {
    if entries.is_empty() {
        return "(no messages)".to_owned();
    }
    let cap = entries.len().saturating_mul(if long { 120 } else { 96 });
    let mut out = String::with_capacity(cap);
    for e in entries {
        let flag = if e.read { ' ' } else { '*' };
        let sender = if e.sender_name.is_empty() { &e.sender_addressing } else { &e.sender_name };
        let dt = fmt_datetime(&e.datetime);
        if long {
            let _ = writeln!(out, "{flag} {}  {}  {}  {}", e.handle, dt, sender, e.subject);
        } else {
            let _ = writeln!(out, "{flag} {}  {}  {}", dt, sender, e.subject);
        }
    }
    out
}

/// Connects, navigates to `folder`, lists messages matching `filter`, and returns the table.
///
/// MAP server-side filters (`originating_address`, `period_begin`, `period_end`) are sent
/// in the request but also applied client-side — the iOS MAP server ignores them.
/// Handle column included when `long` is true; omitted otherwise.
///
/// # Errors
///
/// Returns an error if the connection, folder navigation, or listing request fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    folder: Option<FolderArg>,
    filter: ListMessagesFilter,
    long: bool,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let entries = fetch(&mut client, folder_of(folder), &filter).await?;
    let filtered = apply_filters(
        &entries,
        filter.originating_address.as_deref(),
        filter.period_begin.as_deref(),
        filter.period_end.as_deref(),
    );
    Ok(render(&filtered, long))
}

/// Navigates to `folder` then lists messages — the caller-must-navigate-first contract
/// of [`MapClient::list_messages`], satisfied in one place.
async fn fetch<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    folder: Folder,
    filter: &ListMessagesFilter,
) -> Result<Vec<MessageEntry>, map_core::MapError> {
    client.set_folder(folder).await?;
    client.list_messages(filter).await
}
