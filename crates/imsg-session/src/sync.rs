//! Store-integrated sync: message ingestion and per-folder cursor backfill.

use crate::util::now_ms;
pub use crate::util::{datetime_to_ms, ms_to_display};

use map_core::client::MapClient;
use map_core::folders::Folder;
use store::{Direction, FolderSyncStatus, NewMessage, Store};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::fetch::{fetch_folder, FetchedMessage};

/// Maps a device-read message to a store insert record, stamping `synced_at`.
///
/// `direction` is derived from `sent`; `status` is `1` for read, `0` for unread. The persist
/// boundary — store shape (`NewMessage`) stays out of the shared read path.
fn to_new_message(msg: FetchedMessage, synced_at: i64) -> NewMessage {
    NewMessage {
        map_handle: msg.handle,
        timestamp_ms: msg.timestamp_ms,
        folder: msg.folder,
        direction: if msg.sent { Direction::Sent } else { Direction::Received },
        address: msg.address,
        status: i32::from(msg.read),
        synced_at,
        text: msg.text,
        outgoing_status: None,
    }
}

/// Fetches and upserts all messages in `folder` since the per-folder cursor anchor, paging
/// at 1024 messages per request.
///
/// Reads the cursor before fetching to derive `since_ms` (`None` on first run = full fetch).
/// Tracks the highest `timestamp_ms` seen across all upserted messages; on success writes the
/// cursor with `sync_status = Complete` and `highest_ts` set to that value (or the previous
/// `highest_ts` when no new messages were found). The cursor is not updated on error —
/// the next run retries from the same anchor.
///
/// # Errors
///
/// Returns an error if any MAP operation or store read/write fails.
async fn backfill_folder<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    folder: Folder,
    now: i64,
) -> anyhow::Result<()> {
    let cursor = store.get_cursor(folder.as_str()).await?;
    let since_ms = cursor.as_ref().map(|c| c.highest_ts);
    // Preserve the previous highest_ts when no new messages arrive this run.
    let mut highest_ts_seen = cursor.as_ref().map_or(0, |c| c.highest_ts);
    let folder_str = folder.as_str();
    let is_sent = folder == Folder::Sent;

    for msg in fetch_folder(client, folder, since_ms, now).await? {
        if msg.timestamp_ms > highest_ts_seen {
            highest_ts_seen = msg.timestamp_ms;
        }
        let handle = msg.handle.clone();
        store.upsert(to_new_message(msg, now)).await?;
        // Reconcile: if this Sent message matches a speculatively-created local row
        // that was awaiting confirmation, advance its outgoing_status to sent_confirmed.
        if is_sent {
            store.reconcile_outgoing(&handle).await?;
        }
    }

    // Cursor is written only on full success; errors above exit before reaching this line.
    store.set_cursor(folder_str, now, highest_ts_seen, FolderSyncStatus::Complete).await?;
    Ok(())
}

/// Fetches MAP messages and upserts them into the store using per-folder cursor anchors.
///
/// When `folder_scope` is `None`, all four folders are processed in order: `Inbox`, `Sent`,
/// `Deleted`, `Outbox`. When `Some`, only the specified folder is processed.
///
/// Each folder runs independently: a failure on one folder is logged and skipped so other
/// folders can still advance their cursors. Returns an error if any folder failed.
///
/// # Errors
///
/// Returns the first folder error encountered. Callers should treat a partial failure as
/// "sync incomplete" and re-run to retry the failed folder(s).
pub async fn backfill<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    folder_scope: Option<Folder>,
) -> anyhow::Result<()> {
    let all_folders = [Folder::Inbox, Folder::Sent, Folder::Deleted, Folder::Outbox];
    let single;
    let folders: &[Folder] = if let Some(f) = folder_scope {
        single = [f];
        &single
    } else {
        &all_folders
    };

    let now = now_ms();
    let mut first_err: Option<anyhow::Error> = None;

    for &folder in folders {
        if let Err(e) = backfill_folder(client, store, folder, now).await {
            tracing::warn!("backfill: {} failed — {e:#}", folder.as_str());
            if first_err.is_none() {
                first_err = Some(e);
            }
        }
    }

    first_err.map_or(Ok(()), Err)
}

/// Runs an incremental backfill across all folders using per-folder cursor anchors.
///
/// Each folder's cursor determines the pull boundary; a folder with no cursor triggers a full
/// fetch. This is the sync coordinator's entry point — it is not intended for read commands.
/// Call [`backfill`] directly when a folder scope is needed.
///
/// # Errors
///
/// Returns an error if any folder's backfill fails; see [`backfill`] for continuation semantics.
pub async fn backfill_catch_up<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
) -> anyhow::Result<()> {
    backfill(client, store, None).await
}
