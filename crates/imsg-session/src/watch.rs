//! Live MNS event processing and the watch loop.

use map_core::client::MapClient;
use map_core::folders::Folder;
use map_core::MessageStatus;
use store::{Direction, NewMessage, Store, STATUS_READ};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, watch};

use crate::outbox::drain_outbox;
use crate::sync::backfill_catch_up;
use crate::util::now_ms;
use crate::{EventType, MnsEvent};

/// Maps a MAP folder path segment (or full path like `telecom/msg/inbox`) to a [`Folder`].
///
/// Matches on the last `/`-delimited segment. Returns `None` for unknown folder names.
fn parse_folder(s: &str) -> Option<Folder> {
    // rsplit('/').next() always yields Some on any &str; unwrap_or(s) is a no-op fallback.
    let leaf = s.rsplit('/').next().unwrap_or(s);
    match leaf {
        "inbox" => Some(Folder::Inbox),
        "sent" => Some(Folder::Sent),
        "outbox" => Some(Folder::Outbox),
        "deleted" => Some(Folder::Deleted),
        _ => None,
    }
}

/// Processes a single MNS event against the store.
///
/// `NewMessage` — navigates to the event folder, fetches the body, and upserts.
/// `MessageDeleted` — deletes by handle. `MessageShift` — updates folder.
/// `DeliverySuccess`/`ReadStatusChanged` — marks status read. Other event types are
/// ignored. Missing handle or unknown folder on `NewMessage` are logged and skipped.
///
/// # Errors
///
/// Returns an error if the store write fails. MAP fetch errors on `NewMessage` are propagated.
pub async fn handle_mns_event<T: AsyncRead + AsyncWrite + Unpin>(
    event: &MnsEvent,
    client: &mut MapClient<T>,
    store: &Store,
    now: i64,
) -> anyhow::Result<()> {
    match event.event_type() {
        EventType::NewMessage => {
            let (Some(handle), Some(folder_raw)) = (event.handle(), event.folder()) else {
                tracing::warn!("NewMessage event missing handle or folder — skipped");
                return Ok(());
            };
            let Some(folder) = parse_folder(folder_raw) else {
                tracing::warn!("NewMessage event unknown folder {folder_raw} — skipped");
                return Ok(());
            };
            client.set_folder(folder).await?;
            let bmsg = client.get_message(handle).await?;
            let address = bmsg.originator().map(|o| o.tel.clone()).unwrap_or_default();
            let status = i32::from(matches!(bmsg.status(), MessageStatus::Read));
            let msg = NewMessage {
                map_handle: handle.to_owned(),
                timestamp_ms: now,
                folder: folder_raw.to_owned(),
                direction: Direction::Received,
                address,
                status,
                synced_at: now,
                text: bmsg.envelope().body.text.clone(),
                outgoing_status: None,
            };
            store.upsert(msg).await?;
        }
        EventType::MessageDeleted => {
            if let Some(handle) = event.handle() {
                store.delete_by_handle(handle).await?;
            }
        }
        EventType::MessageShift => {
            if let (Some(handle), Some(folder)) = (event.handle(), event.folder()) {
                store.update_folder(handle, folder).await?;
            }
        }
        EventType::DeliverySuccess | EventType::ReadStatusChanged => {
            if let Some(handle) = event.handle() {
                store.update_status(handle, STATUS_READ).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Runs a catch-up backfill, drains queued outbox entries, then processes live MNS events
/// until cancellation.
///
/// Catch-up uses per-folder cursors — each folder pulls only what it missed since its
/// last successful sync. Queued outbox entries are flushed after backfill so that any
/// message pending from a prior failed send is delivered before the event loop begins.
/// Each `NewMessage` event fetches the body from `client`; other event types only touch
/// the store. Returns when `cancel_rx` is set to `true` or `event_rx` closes.
///
/// # Errors
///
/// Returns an error if the initial backfill, outbox drain, or any store write fails.
pub async fn run_watch<T: AsyncRead + AsyncWrite + Unpin>(
    event_rx: &mut mpsc::Receiver<MnsEvent>,
    client: &mut MapClient<T>,
    store: &Store,
    mut cancel_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    backfill_catch_up(client, store).await?;
    drain_outbox(client, store, now_ms()).await?;
    loop {
        tokio::select! {
            biased;
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() { break; }
            }
            event = event_rx.recv() => {
                let Some(ev) = event else { break; };
                handle_mns_event(&ev, client, store, now_ms()).await?;
            }
        }
    }
    Ok(())
}
