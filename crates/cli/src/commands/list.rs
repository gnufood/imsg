//! `list` subcommand: read a folder live from the device (non-opted) or from the local store
//! (opted-in), and render a message table.

use std::fmt::Write as _;
use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::{BrokerRequest, BrokerResponse, MessageDto};
use session::live::models::LiveMessage;
use session::live::ListFilter;
use store::{MessageRow, Store, STATUS_UNREAD};
use transport::iroh::Endpoint;

use crate::cli::{folder_of, FolderArg};
use crate::commands::{broker, conn, live_footer};

/// Query display options for the `list` subcommand.
pub(crate) struct ListOpts {
    /// Folder to filter; defaults to `inbox` when `None`.
    pub folder: Option<FolderArg>,
    /// When `true`, only unread messages are returned.
    pub unread: bool,
    /// Filter to a single resolved address when `Some`.
    pub from: Option<String>,
    /// Earliest message datetime as a MAP string (`YYYYMMDDTHHMMSS`); ignored if unparseable.
    pub since: Option<String>,
    /// Maximum number of rows; defaults to `1024`.
    pub limit: Option<u16>,
    /// Row offset; defaults to `0`.
    pub offset: Option<u16>,
    /// When `true`, prepend the MAP handle to each output line.
    pub long: bool,
}

/// Reads the folder live from the device and returns the rendered table with a `(live from
/// device)` footer.
///
/// RFCOMM path (`endpoint` is `None`): the broker answers a [`BrokerRequest::ListMessages`].
/// Hub path: a direct MAP connection via [`session::live::list`]. Neither path touches the store.
///
/// # Errors
///
/// Returns an error if the broker call, MAP connection, or device listing fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: ListOpts,
    config_path: Option<&Path>,
) -> Result<String> {
    let folder = folder_of(opts.folder);
    if endpoint.is_none() {
        let rows = list_via_broker(cfg, device, config_path, &opts, folder.as_str()).await?;
        let lines: Vec<MsgLine> = rows.iter().map(MsgLine::from_dto).collect();
        return Ok(live_footer(render_rows(&lines, opts.long)));
    }
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::live::list(&mut client, folder, &to_filter(&opts)).await;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    let rows = result?;
    let lines: Vec<MsgLine> = rows.iter().map(MsgLine::from_live).collect();
    Ok(live_footer(render_rows(&lines, opts.long)))
}

/// Queries the local store and returns the rendered message table with a freshness footer.
///
/// Never opens a Bluetooth connection. If the store has never been synced the table will
/// be empty and the footer prompts the user to run `imsg sync`.
///
/// # Errors
///
/// Returns an error if the store read or `last_sync_at` query fails.
pub(crate) async fn run_store(opts: ListOpts, store: &Store) -> Result<String> {
    let folder_str = folder_of(opts.folder).as_str();
    let since_ms = opts.since.as_deref().and_then(session::sync::datetime_to_ms);
    let rows = store
        .list_messages(
            Some(folder_str),
            opts.unread,
            opts.from.as_deref(),
            since_ms,
            opts.limit.unwrap_or(1024),
            opts.offset.unwrap_or(0),
        )
        .await?;
    let lines: Vec<MsgLine> = rows.iter().map(MsgLine::from_row).collect();
    let mut out = render_rows(&lines, opts.long);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&crate::commands::freshness_line(store.last_sync_at().await?));
    Ok(out)
}

/// Issues a [`BrokerRequest::ListMessages`] and unwraps the [`BrokerResponse::Messages`] frame.
async fn list_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    opts: &ListOpts,
    folder: &str,
) -> Result<Vec<MessageDto>> {
    let req = BrokerRequest::ListMessages {
        folder: Some(folder.to_owned()),
        unread: opts.unread,
        from: opts.from.clone(),
        since: opts.since.clone(),
        limit: opts.limit,
        offset: opts.offset.unwrap_or(0),
    };
    match broker::call(cfg, device, config_path, req).await? {
        BrokerResponse::Messages(rows) => Ok(rows),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// Lowers display options to the in-memory [`ListFilter`] used by the hub path.
fn to_filter(opts: &ListOpts) -> ListFilter {
    ListFilter {
        unread: opts.unread,
        from: opts.from.clone(),
        since_ms: opts.since.as_deref().and_then(session::sync::datetime_to_ms),
        limit: opts.limit,
        offset: opts.offset.unwrap_or(0),
    }
}

/// One rendered message line, sourced from a store row, a live DTO, or a live model.
///
/// `badge` is the store-only delivery state; live sources leave it empty (no device source).
struct MsgLine<'a> {
    unread: bool,
    timestamp_ms: i64,
    address: &'a str,
    text: &'a str,
    handle: &'a str,
    badge: &'static str,
}

impl<'a> MsgLine<'a> {
    fn from_row(row: &'a MessageRow) -> Self {
        Self {
            unread: row.status == STATUS_UNREAD,
            timestamp_ms: row.timestamp_ms,
            address: &row.address,
            text: &row.text,
            handle: &row.map_handle,
            badge: outgoing_badge(row.outgoing_status.as_ref()),
        }
    }

    fn from_dto(m: &'a MessageDto) -> Self {
        Self {
            unread: !m.read,
            timestamp_ms: m.timestamp_ms,
            address: &m.address,
            text: &m.text,
            handle: &m.handle,
            badge: "",
        }
    }

    fn from_live(m: &'a LiveMessage) -> Self {
        Self {
            unread: !m.read,
            timestamp_ms: m.timestamp_ms,
            address: &m.address,
            text: &m.text,
            handle: &m.handle,
            badge: "",
        }
    }
}

/// Renders message lines one per row.
///
/// Each line: unread marker (`*` or ` `), formatted datetime, address, and a body preview
/// truncated at 72 characters. When `long`, the MAP handle is prepended for use with `get`.
/// Returns `"(no messages)"` when `lines` is empty.
fn render_rows(lines: &[MsgLine], long: bool) -> String {
    if lines.is_empty() {
        return "(no messages)".to_owned();
    }
    let cap = lines.len().saturating_mul(if long { 120 } else { 96 });
    let mut out = String::with_capacity(cap);
    for line in lines {
        let flag = if line.unread { '*' } else { ' ' };
        let dt = session::sync::ms_to_display(line.timestamp_ms);
        let preview: String = line.text.chars().take(72).collect();
        if long {
            let _ = writeln!(
                out,
                "{flag} {}  {}  {}  {}{}",
                line.handle, dt, line.address, preview, line.badge,
            );
        } else {
            let _ = writeln!(out, "{flag} {}  {}  {}{}", dt, line.address, preview, line.badge);
        }
    }
    out
}

/// Returns a display badge for a non-`None` `outgoing_status`, or an empty string.
///
/// Badges are appended to message lines to surface delivery state without
/// disrupting the existing column layout.
const fn outgoing_badge(status: Option<&store::OutgoingStatus>) -> &'static str {
    use store::OutgoingStatus;
    match status {
        None => "",
        Some(OutgoingStatus::Queued) => "  [queued]",
        Some(OutgoingStatus::Sending) => "  [sending]",
        Some(OutgoingStatus::SentUnconfirmed) => "  [sent?]",
        Some(OutgoingStatus::SentConfirmed) => "  [confirmed]",
        Some(OutgoingStatus::FailedRetryable) => "  [failed: retryable]",
        Some(OutgoingStatus::FailedPermanent) => "  [failed]",
        Some(OutgoingStatus::Unknown) => "  [unknown]",
    }
}
