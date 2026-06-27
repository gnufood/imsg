//! Outbox drain: push-error classification, single-message send, and outbox flush.

use map_core::{client::MapClient, folders::Folder, MapError};
use store::{Direction, NewMessage, OutboxStatus, OutgoingStatus, Store, STATUS_READ};
use tokio::io::{AsyncRead, AsyncWrite};

/// Classifies a MAP push error into the appropriate outbox and message delivery states.
///
/// Returns `(OutboxStatus, OutgoingStatus)` to be written to the outbox entry and the
/// speculative message row respectively.
///
/// Transport errors map to `Unknown` because the PUT may have been transmitted before
/// the connection dropped; reconciliation against the Sent folder is required to resolve.
/// Server and input rejections map to `FailedPermanent`; all other errors to `FailedRetryable`.
#[must_use]
pub const fn classify_push_error(e: &MapError) -> (OutboxStatus, OutgoingStatus) {
    match e {
        // Transport errors: the PUT may have been sent before the drop — outcome is ambiguous.
        MapError::Transport(_) | MapError::UnexpectedEof => {
            (OutboxStatus::Unknown, OutgoingStatus::Unknown)
        }
        // Definitive rejections: retrying will not help.
        MapError::InvalidInput(_) | MapError::ServerError(_) => {
            (OutboxStatus::Failed, OutgoingStatus::FailedPermanent)
        }
        // OBEX protocol, encoding, or parse errors: transient, worth retrying.
        _ => (OutboxStatus::Failed, OutgoingStatus::FailedRetryable),
    }
}

/// Returns `true` when `e` indicates the RFCOMM/OBEX transport has died and no further MAP
/// operations will succeed on this session.
///
/// `Transport` and `UnexpectedEof` are the only `MapError` variants that represent a dead
/// stream. All others (server rejections, parse errors, encoding failures) leave the session
/// alive. Use [`is_fatal_anyhow`] when working with `anyhow::Error` return values.
#[must_use]
pub const fn is_session_fatal(e: &MapError) -> bool {
    matches!(e, MapError::Transport(_) | MapError::UnexpectedEof)
}

/// Returns `true` when any error in the chain is a fatal MAP transport error.
///
/// Walks the full `anyhow` cause chain, so callers may freely add `.context()` without
/// breaking classification. Only [`MapError::Transport`] and [`MapError::UnexpectedEof`]
/// are considered fatal; all other variants leave the session alive.
#[must_use]
pub fn is_fatal_anyhow(e: &anyhow::Error) -> bool {
    e.chain().filter_map(|cause| cause.downcast_ref::<MapError>()).any(is_session_fatal)
}

/// Records the outbound message in the store, pushes it to the device, and updates the
/// outcome — all as a single atomic sequence.
///
/// Enqueues a `Queued` outbox entry, navigates to the MAP Outbox folder, marks the entry
/// `Sending`, calls `push_message`, then either commits success via `complete_send` or
/// records the classified failure via `resolve` + `update_outgoing_status`. Store errors on
/// the failure path are logged as warnings so the push error is always the returned value.
///
/// Returns the sent confirmation string (`"sent to {number} (handle {handle})"`) on success.
///
/// # Errors
///
/// Returns an error if the store enqueue fails, the MAP folder navigation fails, or the
/// push fails. A `MapError::Transport` or `MapError::UnexpectedEof` root indicates the
/// session is dead; callers should check with [`is_fatal_anyhow`] to decide whether to
/// propagate the failure or surface it as a non-fatal response.
pub async fn send_sms<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    number: &str,
    message: &str,
    now: i64,
) -> anyhow::Result<String> {
    let params = format!("{number}\x1F{message}");
    let (_, outbox_id) = store
        .enqueue_send(
            NewMessage {
                map_handle: String::new(),
                timestamp_ms: now,
                folder: Folder::Sent.as_str().to_owned(),
                direction: Direction::Sent,
                address: number.to_owned(),
                status: STATUS_READ,
                synced_at: now,
                text: message.to_owned(),
                outgoing_status: Some(OutgoingStatus::Queued),
            },
            "send_sms",
            &params,
            now,
        )
        .await?;
    let placeholder = format!("local:{outbox_id}");

    client.set_folder(Folder::Outbox).await?;
    store.resolve(outbox_id, OutboxStatus::Sending, now, None).await?;

    match client.push_message(number, message).await {
        Ok(handle) => {
            store.complete_send(outbox_id, &placeholder, &handle, now).await?;
            Ok(format!("sent to {number} (handle {handle})"))
        }
        Err(e) => {
            let (outbox_status, outgoing_status) = classify_push_error(&e);
            if let Err(se) = store.resolve(outbox_id, outbox_status, now, Some(e.to_string())).await
            {
                tracing::warn!("resolve outbox {outbox_id}: {se}");
            }
            if let Err(se) = store.update_outgoing_status(&placeholder, outgoing_status).await {
                tracing::warn!("update outgoing status {placeholder}: {se}");
            }
            // Return MapError as root so callers can classify via is_fatal_anyhow.
            Err(anyhow::Error::from(e))
        }
    }
}

/// Pushes an outgoing SMS to the device without recording it in the store.
///
/// The non-opted-in `send` path: navigates to the MAP Outbox folder and pushes, returning the same
/// confirmation string as [`send_sms`]. Fire-and-forget — no outbox row, so no delivery tracking or
/// retry; a transient failure surfaces to the caller to re-send. No store access.
///
/// # Errors
///
/// Returns an error if the MAP folder navigation or push fails. A `MapError::Transport` or
/// `MapError::UnexpectedEof` root indicates a dead session; classify with [`is_fatal_anyhow`].
pub async fn push_sms<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    number: &str,
    message: &str,
) -> anyhow::Result<String> {
    client.set_folder(Folder::Outbox).await?;
    let handle = client.push_message(number, message).await?;
    Ok(format!("sent to {number} (handle {handle})"))
}

/// Fetches all `queued` outbox entries and pushes each to the device, updating the store
/// with the outcome.
///
/// Navigates the MAP client to the Outbox folder once before iterating. Each entry is
/// processed independently: a push failure on one entry is logged and does not abort the
/// rest. Entries with unparseable payloads are skipped with a warning and left `queued`.
///
/// The payload format is `"{number}\x1F{message}"` as written by `send::run`.
///
/// # Errors
///
/// Returns an error if `store.pending()` fails or navigating to the Outbox folder fails.
/// Individual push failures are recorded in the store and do not propagate.
pub async fn drain_outbox<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    now: i64,
) -> anyhow::Result<()> {
    let pending = store.pending().await?;
    if pending.is_empty() {
        return Ok(());
    }
    client.set_folder(Folder::Outbox).await?;
    for entry in pending {
        process_entry(client, store, entry, now).await;
    }
    Ok(())
}

/// Parses and pushes a single outbox entry, recording the outcome in the store.
///
/// Entries with unparseable payloads are skipped with a warning and left `queued`.
/// Push and store errors are logged but do not propagate — callers continue with remaining entries.
async fn process_entry<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    entry: store::OutboxRow,
    now: i64,
) {
    let Some((number, message)) = entry.payload.split_once('\x1F') else {
        tracing::warn!("drain_outbox: entry {} has unparseable payload — skipped", entry.id);
        return;
    };
    let placeholder = format!("local:{}", entry.id);
    match client.push_message(number, message).await {
        Ok(handle) => record_send_ok(store, entry.id, &placeholder, &handle, now).await,
        Err(e) => record_send_err(store, entry.id, &placeholder, &e, now).await,
    }
}

/// Records a successful MAP push: marks the outbox entry sent and links the remote handle.
///
/// Store failures are logged and swallowed — the push already succeeded on the device.
async fn record_send_ok(store: &Store, entry_id: i64, placeholder: &str, handle: &str, now: i64) {
    store.complete_send(entry_id, placeholder, handle, now).await.unwrap_or_else(|e| {
        tracing::warn!("drain_outbox: store update failed for entry {entry_id}: {e:#}");
    });
}

/// Records a failed MAP push: classifies the error and updates both the outbox and message rows.
///
/// Store failures are logged and swallowed to ensure all update attempts run regardless.
async fn record_send_err(
    store: &Store,
    entry_id: i64,
    placeholder: &str,
    e: &map_core::MapError,
    now: i64,
) {
    let (outbox_status, outgoing_status) = classify_push_error(e);
    tracing::warn!("drain_outbox: push failed for entry {entry_id}: {e}");
    let err_str = e.to_string();
    store.resolve(entry_id, outbox_status, now, Some(err_str)).await.unwrap_or_else(|se| {
        tracing::warn!("drain_outbox: resolve failed for entry {entry_id}: {se:#}");
    });
    store.update_outgoing_status(placeholder, outgoing_status).await.unwrap_or_else(|se| {
        tracing::warn!("drain_outbox: status update failed for entry {entry_id}: {se:#}");
    });
}
