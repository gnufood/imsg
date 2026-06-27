//! MAP operation handlers and the session-error → wire-reason mapping.
//!
//! Each handler borrows `MapClient<T>` and `Store`, performs one MAP operation, and returns a
//! [`BrokerResponse`]. Fatal transport errors propagate as `Err` so the actor reconnects;
//! application-level failures (server rejections, unknown handles) are returned as
//! [`BrokerResponse::Failed`] with [`Reason::OperationFailed`]. This module is the boundary that
//! maps `imsg-session` error types to the serde-only [`Reason`] enum.

use anyhow::Result;
use ipc::{BrokerResponse, Reason};
use map_core::client::MapClient;
use map_core::folders::Folder;
use map_core::MessageStatus;
use session::{Disposition, SessionError};
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};

use super::dto::{to_body_dto, to_message_dto, to_thread_dto};

/// Maps a session-establishment failure to the action-oriented wire [`Reason`].
///
/// Permanent failures (auth/pairing/wrong channel) become [`Reason::ConnectionRefused`];
/// transient ones (link timeout/reset) become [`Reason::DeviceUnreachable`].
pub(in crate::runtime::actor) fn connect_reason(e: &SessionError) -> Reason {
    match session::classify(e) {
        Disposition::Permanent => Reason::ConnectionRefused,
        Disposition::Transient => Reason::DeviceUnreachable,
    }
}

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
/// # Errors
///
/// Returns [`BrokerResponse::Failed`] for unknown folder names or non-fatal MAP
/// errors. Propagates fatal transport errors.
pub(in crate::runtime::actor) async fn do_delete<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    handle: String,
    folder: String,
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
        Ok(()) => Ok(BrokerResponse::Text(format!("deleted {handle}"))),
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

/// Lists a folder live and returns lean message DTOs, applying the client-side filters.
///
/// No store write or cursor advance. An unknown folder name is a non-fatal
/// [`Reason::OperationFailed`]; fatal transport errors propagate as `Err`.
pub(in crate::runtime::actor) async fn do_live_list<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    folder: Option<String>,
    unread: bool,
    from: Option<String>,
    since: Option<String>,
    limit: Option<u16>,
    offset: u16,
) -> Result<BrokerResponse> {
    let folder_val = if let Some(name) = folder.as_deref() {
        let Some(f) = parse_folder(name) else {
            let reason = Reason::OperationFailed(format!("unknown folder: {name}"));
            return Ok(BrokerResponse::Failed(reason));
        };
        f
    } else {
        Folder::Inbox
    };
    let filter = session::live::ListFilter {
        unread,
        from,
        since_ms: since.as_deref().and_then(session::sync::datetime_to_ms),
        limit,
        offset,
    };
    match session::live::list(client, folder_val, &filter).await {
        Ok(msgs) => Ok(BrokerResponse::Messages(msgs.into_iter().map(to_message_dto).collect())),
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Fetches one message body live by handle and returns its DTO. No store write.
///
/// Propagates fatal transport errors; wraps non-fatal errors in [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_live_get<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    handle: String,
) -> Result<BrokerResponse> {
    match session::live::get(client, handle).await {
        Ok(body) => Ok(BrokerResponse::Body(to_body_dto(body))),
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Aggregates live Inbox+Sent listings into per-contact thread DTOs. No store write.
///
/// Propagates fatal transport errors; wraps non-fatal errors in [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_live_threads<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
) -> Result<BrokerResponse> {
    match session::live::threads(client).await {
        Ok(threads) => {
            Ok(BrokerResponse::Threads(threads.into_iter().map(to_thread_dto).collect()))
        }
        Err(e) if session::outbox::is_fatal_anyhow(&e) => Err(e),
        Err(e) => Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string()))),
    }
}

/// Marks a MAP message read on the device only, leaving the store untouched.
///
/// The non-opted-in `get --read` path: the message is not persisted here, so there is no store row
/// to edit. Propagates fatal transport errors; wraps non-fatal errors in [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_live_mark_read<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    handle: String,
) -> Result<BrokerResponse> {
    match client.set_message_status_read(&handle, MessageStatus::Read).await {
        Ok(()) => Ok(BrokerResponse::Ok),
        Err(e) => {
            let e = anyhow::Error::new(e);
            if session::outbox::is_fatal_anyhow(&e) {
                Err(e)
            } else {
                Ok(BrokerResponse::Failed(Reason::OperationFailed(e.to_string())))
            }
        }
    }
}

/// Pushes an outgoing SMS to the device only, leaving the store untouched.
///
/// The non-opted-in `send` path: fire-and-forget, no outbox row. Propagates fatal transport errors;
/// wraps non-fatal errors in [`BrokerResponse::Failed`].
pub(in crate::runtime::actor) async fn do_live_send<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    number: String,
    message: String,
) -> Result<BrokerResponse> {
    match session::outbox::push_sms(client, &number, &message).await {
        Ok(confirmation) => Ok(BrokerResponse::Text(confirmation)),
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
