//! `get` subcommand: fetch one message body live from the device (non-opted) or from the
//! local store (opted-in).

use std::fmt::Write as _;
use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::{BodyDto, BrokerRequest, BrokerResponse};
use map_core::MessageStatus;
use session::live::models::{Direction as LiveDirection, LiveBody};
use store::{Direction, MessageRow, Store, STATUS_READ, STATUS_UNREAD};
use transport::iroh::Endpoint;

use crate::commands::{broker, conn, live_footer};

/// Fetches one message body live from the device, optionally marking it read.
///
/// RFCOMM path (`endpoint` is `None`): the broker answers a [`BrokerRequest::GetMessage`], and
/// `mark_read` issues a best-effort device-only [`BrokerRequest::MarkReadDevice`]. Hub path: a
/// direct MAP connection via [`session::live::get`], with device-side mark-read. Neither path
/// touches the store; mark-read failures are logged, never propagated.
///
/// # Errors
///
/// Returns an error if the broker call, MAP connection, or body fetch fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    handle: String,
    mark_read: bool,
    config_path: Option<&Path>,
) -> Result<String> {
    if endpoint.is_none() {
        let body = get_via_broker(cfg, device, config_path, handle.clone()).await?;
        let out = render_row(&BodyView::from_dto(&body));
        if mark_read && !body.read {
            broker::call(cfg, device, config_path, BrokerRequest::MarkReadDevice { handle })
                .await
                .ok();
        }
        return Ok(live_footer(out));
    }
    let body = get_via_map(cfg, endpoint, device, handle, mark_read).await?;
    Ok(live_footer(render_row(&BodyView::from_live(&body))))
}

/// Fetches one body over a direct MAP connection, marking it read on the device when requested.
///
/// Mark-read and disconnect failures are logged, never propagated. The connection is always
/// closed before returning.
async fn get_via_map(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    handle: String,
    mark_read: bool,
) -> Result<LiveBody> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::live::get(&mut client, handle.clone()).await;
    if let Ok(body) = &result {
        if mark_read && !body.read {
            if let Err(e) = client.set_message_status_read(&handle, MessageStatus::Read).await {
                tracing::warn!("failed to mark {handle} read on device: {e}");
            }
        }
    }
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    result
}

/// Retrieves one message from the local store and returns it with a freshness footer.
///
/// Never opens a Bluetooth connection. If `mark_read` is `true`, the `status` column is
/// updated locally; device-side mark-read is deferred until the next `imsg sync` run.
///
/// # Errors
///
/// Returns an error if the store read fails or the handle is not present in the store.
pub(crate) async fn run_store(handle: String, mark_read: bool, store: &Store) -> Result<String> {
    let row = store
        .get_by_handle(&handle)
        .await?
        .ok_or_else(|| anyhow::anyhow!("message {handle} not found in local store"))?;
    if mark_read && row.status == STATUS_UNREAD {
        // Device-side mark-read is deferred; update local store only.
        if let Err(e) = store.update_status(&handle, STATUS_READ).await {
            tracing::warn!("failed to update read status in store for {handle}: {e}");
        }
    }
    let mut out = render_row(&BodyView::from_row(&row));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&crate::commands::freshness_line(store.last_sync_at().await?));
    Ok(out)
}

/// Issues a [`BrokerRequest::GetMessage`] and unwraps the [`BrokerResponse::Body`] frame.
async fn get_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    handle: String,
) -> Result<BodyDto> {
    match broker::call(cfg, device, config_path, BrokerRequest::GetMessage { handle }).await? {
        BrokerResponse::Body(body) => Ok(body),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// One rendered message body, sourced from a store row, a live DTO, or a live model.
///
/// `date` is `None` for live bodies — a bMessage carries no datetime — and the `Date:` line is
/// dropped in that case.
struct BodyView<'a> {
    label: &'static str,
    address: &'a str,
    date: Option<String>,
    folder: &'a str,
    read: bool,
    text: &'a str,
}

impl<'a> BodyView<'a> {
    fn from_row(row: &'a MessageRow) -> Self {
        Self {
            label: dir_label(matches!(row.direction, Direction::Sent)),
            address: &row.address,
            date: Some(session::sync::ms_to_display(row.timestamp_ms)),
            folder: &row.folder,
            read: row.status == STATUS_READ,
            text: &row.text,
        }
    }

    fn from_dto(b: &'a BodyDto) -> Self {
        Self {
            label: dir_label(matches!(b.direction, ipc::Direction::Sent)),
            address: &b.address,
            date: None,
            folder: &b.folder,
            read: b.read,
            text: &b.text,
        }
    }

    fn from_live(b: &'a LiveBody) -> Self {
        Self {
            label: dir_label(matches!(b.direction, LiveDirection::Sent)),
            address: &b.address,
            date: None,
            folder: &b.folder,
            read: b.read,
            text: &b.text,
        }
    }
}

const fn dir_label(sent: bool) -> &'static str {
    if sent {
        "To"
    } else {
        "From"
    }
}

/// Formats a message body as a header block then body. Omits the `Date:` line when `date` is `None`.
fn render_row(v: &BodyView) -> String {
    let status = if v.read { "read" } else { "unread" };
    let mut out = String::with_capacity(v.text.len().saturating_add(64));
    let _ = writeln!(out, "{}: {}", v.label, v.address);
    if let Some(dt) = &v.date {
        let _ = writeln!(out, "Date: {dt}");
    }
    let _ = writeln!(out, "Folder: {}", v.folder);
    let _ = writeln!(out, "Status: {status}");
    let _ = writeln!(out);
    let _ = write!(out, "{}", v.text);
    out
}
