//! Live-query orchestration: `list`/`threads`/`get` read straight from the device and return
//! lean models, with no store writes and no cursor advances.
//!
//! Both read arms (the in-process CLI path and the broker path) call these. `map_core` protocol
//! types (`BMessage`, `MessageEntry`) are normalized here and never cross outward — only the lean
//! models in [`models`] do.

pub mod models;

use std::collections::HashMap;

use map_core::client::MapClient;
use map_core::folders::Folder;
use map_core::messages::{ListMessagesFilter, MessageEntry, ReadStatus};
use map_core::{BMessage, MessageStatus};
use store::PhoneField;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::fetch::{fetch_folder, list_folder};
use crate::util::{datetime_to_ms, now_ms};
use models::{Direction, LiveBody, LiveFolder, LiveMessage, LiveThread};

/// Filters for a live [`list`], mirroring `store::list_messages` semantics.
///
/// `unread` is pushed down to the device via `FilterReadStatus`. `from`/`since_ms`/`limit`/
/// `offset` are applied in memory over the device's listing window: the device returns its whole
/// fixed window regardless of offset, so paging is meaningless against it.
#[derive(Debug, Default)]
pub struct ListFilter {
    /// Keep only unread messages.
    pub unread: bool,
    /// Keep only messages whose resolved address equals this value exactly.
    pub from: Option<String>,
    /// Keep only messages at or after this epoch-millisecond datetime.
    pub since_ms: Option<i64>,
    /// Maximum rows after `offset`; `None` keeps the rest of the window.
    pub limit: Option<u16>,
    /// Rows to skip from the front of the newest-first window.
    pub offset: u16,
}

/// Lists `folder` live and returns lean messages newest-first, after applying `filter`.
///
/// `unread` is filtered device-side via `FilterReadStatus`, so an unread-only request never
/// fetches a read message's body. Remaining filters, sort, and windowing happen in memory over
/// whatever the device returns. No store access, no cursor advance.
///
/// # Errors
///
/// Returns an error if any MAP listing or body fetch fails.
pub async fn list<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    folder: Folder,
    filter: &ListFilter,
) -> anyhow::Result<Vec<LiveMessage>> {
    let template =
        ListMessagesFilter { read_status: read_status_for(filter), ..Default::default() };
    let mut msgs: Vec<LiveMessage> = fetch_folder(client, folder, None, now_ms(), &template)
        .await?
        .into_iter()
        .map(|m| LiveMessage {
            handle: m.handle,
            timestamp_ms: m.timestamp_ms,
            address: canonical(&m.address),
            folder: m.folder,
            read: m.read,
            text: m.text,
        })
        .filter(|m| keep(m, filter))
        .collect();
    msgs.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    Ok(window(msgs, filter))
}

fn read_status_for(f: &ListFilter) -> Option<ReadStatus> {
    f.unread.then_some(ReadStatus::Unread)
}

/// Best-effort E.164 for grouping/matching; the raw form when it can't be resolved.
fn canonical(raw: &str) -> String {
    PhoneField::new(raw, None).display().to_owned()
}

fn keep(m: &LiveMessage, f: &ListFilter) -> bool {
    // `m.address` is already canonical; normalise the filter value so formatting differences
    // don't cause a miss.
    if f.from.as_deref().is_some_and(|addr| canonical(addr) != m.address) {
        return false;
    }
    f.since_ms.is_none_or(|since| m.timestamp_ms >= since)
}

fn window(msgs: Vec<LiveMessage>, f: &ListFilter) -> Vec<LiveMessage> {
    let rest = msgs.into_iter().skip(usize::from(f.offset));
    match f.limit {
        Some(n) => rest.take(usize::from(n)).collect(),
        None => rest.collect(),
    }
}

/// Aggregates Inbox and Sent listings into per-contact thread summaries, newest-first.
///
/// Listings only — no bodies are fetched. Mirrors `store::threads`: `total` counts all messages,
/// `unread` counts received-and-unread, `latest_ms` is the max datetime, empty-address entries are
/// dropped. Counts are approximate (device window, not full corpus). No store access.
///
/// # Errors
///
/// Returns an error if either folder listing fails.
pub async fn threads<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
) -> anyhow::Result<Vec<LiveThread>> {
    let mut acc: HashMap<String, LiveThread> = HashMap::new();
    for folder in [Folder::Inbox, Folder::Sent] {
        for entry in list_folder(client, folder, &ListMessagesFilter::default()).await? {
            accumulate(&mut acc, &entry);
        }
    }
    let mut threads: Vec<LiveThread> = acc.into_values().collect();
    threads.sort_by(|a, b| b.latest_ms.cmp(&a.latest_ms));
    Ok(threads)
}

/// Lists the device's MAP message folders under `telecom/msg`, in device-reported document order.
///
/// Navigates to `telecom/msg` first, so the result is the message-folder level rather than
/// whatever directory the session was left in. Leaves the client parked there. No store access.
///
/// # Errors
///
/// Returns an error if any SETPATH fails, the device rejects the listing, or the listing XML is
/// malformed.
pub async fn folders<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
) -> anyhow::Result<Vec<LiveFolder>> {
    let listing = client.list_message_folders().await?;
    Ok(listing.folders().iter().map(|f| LiveFolder { name: f.name().to_owned() }).collect())
}

fn accumulate(acc: &mut HashMap<String, LiveThread>, entry: &MessageEntry) {
    let address = peer_address(entry);
    if address.is_empty() {
        return;
    }
    // Group on the canonical form so formatting variants collapse into one thread.
    let address = canonical(&address);
    let ts = datetime_to_ms(&entry.datetime).unwrap_or(0);
    let t = acc.entry(address.clone()).or_insert_with(|| LiveThread {
        address,
        latest_ms: ts,
        total: 0,
        unread: 0,
    });
    t.total = t.total.saturating_add(1);
    t.unread = t.unread.saturating_add(u32::from(!entry.sent && !entry.read));
    if ts > t.latest_ms {
        t.latest_ms = ts;
    }
}

fn peer_address(entry: &MessageEntry) -> String {
    if entry.sent {
        entry.recipient_addressing.clone()
    } else {
        entry.sender_addressing.clone()
    }
}

/// Fetches one message body live by `handle` and normalizes it to a [`LiveBody`].
///
/// `direction` is derived from the bMessage folder (containing `sent`/`outbox` ⇒ [`Direction::Sent`]);
/// the address is the originator for received messages and the first recipient for sent. No
/// timestamp — a `BMessage` carries no datetime. No store access.
///
/// # Errors
///
/// Returns an error if the MAP `GetMessage` fails.
pub async fn get<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    handle: String,
) -> anyhow::Result<LiveBody> {
    let bmsg = client.get_message(&handle).await?;
    Ok(to_live_body(handle, &bmsg))
}

fn to_live_body(handle: String, bmsg: &BMessage) -> LiveBody {
    let direction = direction_of(bmsg.folder());
    let address = match direction {
        Direction::Sent => bmsg.envelope().recipients.first().map(|v| v.tel.clone()),
        Direction::Received => bmsg.originator().map(|v| v.tel.clone()),
    }
    .unwrap_or_default();
    LiveBody {
        handle,
        direction,
        address: canonical(&address),
        folder: bmsg.folder().to_owned(),
        read: matches!(bmsg.status(), MessageStatus::Read),
        text: bmsg.envelope().body.text.clone(),
    }
}

fn direction_of(folder: &str) -> Direction {
    let f = folder.to_ascii_lowercase();
    if f.contains("sent") || f.contains("outbox") {
        Direction::Sent
    } else {
        Direction::Received
    }
}

#[cfg(test)]
mod tests;
