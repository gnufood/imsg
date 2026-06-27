//! `threads` subcommand: aggregate per-contact threads live from the device (non-opted) or
//! from the local store (opted-in).

use std::fmt::Write as _;
use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::{BrokerRequest, BrokerResponse, ThreadDto};
use session::live::models::LiveThread;
use store::{OutgoingStatus, Store, ThreadRow};
use transport::iroh::Endpoint;

use crate::commands::{broker, conn, live_footer};

/// Returns a per-contact thread summary read live from the device, sorted most-recent-first.
///
/// RFCOMM path (`endpoint` is `None`): the broker answers a [`BrokerRequest::Threads`]. Hub path:
/// a direct MAP connection via [`session::live::threads`]. Counts are approximate (device window,
/// not full corpus) and carry no delivery badge. Neither path touches the store.
///
/// # Errors
///
/// Returns an error if the broker call, MAP connection, or device listing fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    config_path: Option<&Path>,
) -> Result<String> {
    if endpoint.is_none() {
        let rows = threads_via_broker(cfg, device, config_path).await?;
        let lines: Vec<ThreadLine> = rows.iter().map(ThreadLine::from_dto).collect();
        return Ok(live_footer(render_thread_rows(&lines)));
    }
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    let result = session::live::threads(&mut client).await;
    if let Err(e) = client.disconnect().await {
        tracing::warn!("MAP disconnect failed: {e}");
    }
    let rows = result?;
    let lines: Vec<ThreadLine> = rows.iter().map(ThreadLine::from_live).collect();
    Ok(live_footer(render_thread_rows(&lines)))
}

/// Returns per-contact thread summaries from the local store with a freshness footer.
///
/// Never opens a Bluetooth connection.
///
/// # Errors
///
/// Returns an error if the store read or `last_sync_at` query fails.
pub(crate) async fn run_store(store: &Store) -> Result<String> {
    let rows = store.threads().await?;
    let lines: Vec<ThreadLine> = rows.iter().map(ThreadLine::from_row).collect();
    let mut out = render_thread_rows(&lines);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&crate::commands::freshness_line(store.last_sync_at().await?));
    Ok(out)
}

/// Issues a [`BrokerRequest::Threads`] and unwraps the [`BrokerResponse::Threads`] frame.
async fn threads_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
) -> Result<Vec<ThreadDto>> {
    match broker::call(cfg, device, config_path, BrokerRequest::Threads).await? {
        BrokerResponse::Threads(rows) => Ok(rows),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// One rendered thread summary line, sourced from a store row, a live DTO, or a live model.
///
/// `badge` is the store-only delivery state of the latest message; live sources leave it `None`
/// (no device source).
struct ThreadLine<'a> {
    address: &'a str,
    latest_ms: i64,
    total: i64,
    unread: i64,
    badge: Option<&'static str>,
}

impl<'a> ThreadLine<'a> {
    fn from_row(t: &'a ThreadRow) -> Self {
        Self {
            address: &t.address,
            latest_ms: t.latest_ms,
            total: t.total,
            unread: t.unread,
            badge: outgoing_badge(t.latest_outgoing_status.as_ref()),
        }
    }

    fn from_dto(t: &'a ThreadDto) -> Self {
        Self {
            address: &t.address,
            latest_ms: t.latest_ms,
            total: i64::from(t.total),
            unread: i64::from(t.unread),
            badge: None,
        }
    }

    fn from_live(t: &'a LiveThread) -> Self {
        Self {
            address: &t.address,
            latest_ms: t.latest_ms,
            total: i64::from(t.total),
            unread: i64::from(t.unread),
            badge: None,
        }
    }
}

/// Renders thread lines as one summary line each.
///
/// Returns `"(no threads)"` when `lines` is empty.
fn render_thread_rows(lines: &[ThreadLine]) -> String {
    if lines.is_empty() {
        return "(no threads)".to_owned();
    }
    let mut out = String::with_capacity(lines.len().saturating_mul(64));
    for line in lines {
        let dt = session::sync::ms_to_display(line.latest_ms);
        let _ = write!(out, "{}  {} messages", line.address, line.total);
        if line.unread > 0 {
            let _ = write!(out, "  {} unread", line.unread);
        }
        let _ = write!(out, "  (latest: {dt})");
        if let Some(badge) = line.badge {
            let _ = write!(out, "  {badge}");
        }
        let _ = writeln!(out);
    }
    out
}

/// Returns a short delivery badge for the latest message's outgoing status, or `None`.
const fn outgoing_badge(status: Option<&OutgoingStatus>) -> Option<&'static str> {
    match status {
        None => None,
        Some(OutgoingStatus::Queued) => Some("[queued]"),
        Some(OutgoingStatus::Sending) => Some("[sending]"),
        Some(OutgoingStatus::SentUnconfirmed) => Some("[sent?]"),
        Some(OutgoingStatus::SentConfirmed) => Some("[confirmed]"),
        Some(OutgoingStatus::FailedRetryable) => Some("[failed: retryable]"),
        Some(OutgoingStatus::FailedPermanent) => Some("[failed]"),
        Some(OutgoingStatus::Unknown) => Some("[unknown]"),
    }
}
