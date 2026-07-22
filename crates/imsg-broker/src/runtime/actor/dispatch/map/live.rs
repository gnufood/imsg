//! Live (non-opted-in) MAP read/write ops — no store write or cursor advance. Split out of
//! `super` to keep that module under the size ceiling.

use anyhow::Result;
use ipc::{BrokerResponse, Reason};
use map_core::client::MapClient;
use map_core::folders::Folder;
use map_core::MessageStatus;
use tokio::io::{AsyncRead, AsyncWrite};

use super::super::super::dto::{to_body_dto, to_message_dto, to_thread_dto};
use super::parse_folder;

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
