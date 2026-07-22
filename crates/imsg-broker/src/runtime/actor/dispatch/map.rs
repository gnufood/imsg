//! Store-backed MAP operation handlers; the live (non-opted-in) equivalents are in [`live`].
//!
//! Each handler borrows `MapClient<T>` and `Store`, performs one MAP operation, and returns a
//! [`BrokerResponse`]. Fatal transport errors propagate as `Err` so the actor reconnects;
//! application-level failures (server rejections, unknown handles) are returned as
//! [`BrokerResponse::Failed`] with [`Reason::OperationFailed`]. This module is the boundary that
//! maps `imsg-session` error types to the serde-only [`Reason`] enum. [`do_delete`] additionally
//! borrows the actor's `watch_tx` to fan its outcome out to `Watch` subscribers.

use anyhow::Result;
use ipc::{BrokerResponse, EventType, Reason, WatchEvent};
use map_core::client::MapClient;
use map_core::folders::Folder;
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::broadcast;

mod live;

pub(in crate::runtime::actor) use live::{
    do_live_get, do_live_list, do_live_mark_read, do_live_send, do_live_threads,
};

/// Drains the outbox then backfills MAP folders.
///
/// `folder` scopes the backfill to one folder (lowercase IPC name, e.g. `"sent"`); `None`
/// syncs all four standard folders. An unrecognised name is rejected as a non-fatal failure.
/// The outbox drain always runs regardless of scope.
///
/// # Errors
///
/// Propagates fatal transport errors; wraps non-fatal MAP errors in
/// [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_sync<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    folder: Option<String>,
) -> Result<BrokerResponse> {
    let scope = match folder.as_deref() {
        None => None,
        Some(name) => match parse_folder(name) {
            Some(f) => Some(f),
            None => {
                return Ok(BrokerResponse::Failed(Reason::OperationFailed(format!(
                    "unknown folder: {name}"
                ))));
            }
        },
    };
    let now = session::util::now_ms();
    if let Err(e) = session::outbox::drain_outbox(client, store, now).await {
        if session::outbox::is_fatal_anyhow(&e) {
            return Err(e);
        }
        return Ok(BrokerResponse::Failed(Reason::OperationFailed(format!("outbox drain: {e}"))));
    }
    match session::sync::backfill(client, store, scope).await {
        Ok(()) => Ok(BrokerResponse::Text("sync complete".to_owned())),
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Pushes an outgoing SMS through the MAP outbox.
///
/// # Errors
///
/// Propagates fatal transport errors; wraps send failures in [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_send<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    number: String,
    message: String,
) -> Result<BrokerResponse> {
    let now = session::util::now_ms();
    match session::outbox::send_sms(client, store, &number, &message, now).await {
        Ok(confirmation) => Ok(BrokerResponse::Text(confirmation)),
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Deletes a MAP message by handle and folder name.
///
/// On success, fans a `MessageDeleted` [`WatchEvent`] out to `Watch` subscribers — unlike `Send`,
/// a delete has no MNS-driven confirmation to fall back on, so this is the only signal any other
/// client gets that it happened.
///
/// # Errors
///
/// Returns [`BrokerResponse::Failed`] for unknown folder names or non-fatal MAP
/// errors. Propagates fatal transport errors.
pub(in crate::runtime::actor) async fn do_delete<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    handle: String,
    folder: String,
    watch_tx: &broadcast::Sender<WatchEvent>,
) -> Result<BrokerResponse> {
    let Some(folder_val) = parse_folder(&folder) else {
        return Ok(BrokerResponse::Failed(Reason::OperationFailed(format!(
            "unknown folder: {folder}"
        ))));
    };
    let result = async {
        client.set_folder(folder_val).await?;
        client.set_message_status_deleted(&handle, true).await?;
        store.delete_by_handle(&handle).await?;
        anyhow::Ok(())
    }
    .await;
    match result {
        Ok(()) => {
            let _ = watch_tx.send(WatchEvent {
                event_type: EventType::MessageDeleted,
                handle: Some(handle.clone()),
                folder: Some(folder),
                old_folder: None,
                msg_type: None,
                datetime: None,
            });
            Ok(BrokerResponse::Text(format!("deleted {handle}")))
        }
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Runs an incremental catch-up backfill without draining the outbox.
///
/// # Errors
///
/// Propagates fatal transport errors; wraps non-fatal errors in [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_backfill<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
) -> Result<BrokerResponse> {
    match session::sync::backfill_catch_up(client, store).await {
        Ok(()) => Ok(BrokerResponse::Ok),
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Maps a lowercase IPC folder name to the MAP [`Folder`] enum.
fn parse_folder(s: &str) -> Option<Folder> {
    match s {
        "inbox" => Some(Folder::Inbox),
        "sent" => Some(Folder::Sent),
        "outbox" => Some(Folder::Outbox),
        "deleted" => Some(Folder::Deleted),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
