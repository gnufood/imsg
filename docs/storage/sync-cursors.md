# Sync Cursors

Sync cursors track the progress of message synchronization for each folder independently. This design replaces an earlier approach that used a single global timestamp as the anchor for all sync operations, and it enables several important capabilities: partial folder syncs, independent folder resynchronization, and a reliable freshness signal for read operations.

## The Problem with a Global Cursor

In the original schema (V1), synchronization progress was tracked as a single value in the `meta` table. When a sync operation completed, it would update this global anchor to the current time, indicating that all folders had been synchronized up to that point. This approach is simple but breaks down in practical use cases:

- **Partial syncs corrupt the global state.** If a user runs `imsg sync --folder inbox` to synchronize only their inbox, the global cursor advances. Subsequent full syncs would incorrectly assume that other folders (sent, deleted, draft) are also up to date, leading to missed messages.

- **No isolation between folders.** A failed sync on one folder would roll back the global cursor, potentially causing the next sync to re-fetch messages that were already successfully synchronized in other folders.

- **No meaningful freshness signal.** The global cursor tells you only that *some* folder was synced at a particular time, not that *all* folders are current. There's no way to answer the question "how fresh is my data?"

## Per-Folder Cursor Design

The V2 migration introduced a `folder_cursors` table that stores sync state for each MAP folder independently:

```sql
CREATE TABLE folder_cursors (
    folder       TEXT    NOT NULL PRIMARY KEY,
    last_sync_at INTEGER NOT NULL,
    highest_ts   INTEGER NOT NULL,
    sync_status  TEXT    NOT NULL DEFAULT 'never'
);
```

Each row tracks three pieces of information:

- **`folder`**: The MAP folder leaf name (e.g., `"inbox"`, `"sent"`, `"telecom/msg/deleted"`).
- **`last_sync_at`**: Milliseconds since Unix epoch when the last sync operation for this folder completed successfully.
- **`highest_ts`**: The highest `timestamp_ms` observed during the most recent sync. This value becomes the lower bound for the next pull operation, ensuring that incremental syncs fetch only new messages.
- **`sync_status`**: An enumerated state indicating whether the folder has never been synced, is currently syncing, completed successfully, or failed.

The `sync_status` field uses the `FolderSyncStatus` enum with four variants:

```rust
pub enum FolderSyncStatus {
    /// No sync has been attempted for this folder yet.
    Never,
    /// A sync is currently in progress.
    Syncing,
    /// Last sync completed successfully.
    Complete,
    /// Last sync ended with an error; `highest_ts` reflects the last successful boundary.
    Failed,
}
```

This state machine allows the sync worker to distinguish between "never synced" and "failed mid-sync," which matters for retry logic and for presenting accurate status to users.

## Cursor Operations

The `Store` type provides three public operations for working with cursors:

### Reading a Single Folder's Cursor

```rust
pub async fn get_cursor(&self, folder: &str) -> Result<Option<FolderCursor>, Error>
```

This retrieves the complete cursor state for one folder, or `None` if the folder has never been synchronized. The caller uses this to determine where to start pulling messages on the next sync cycle.

### Updating a Cursor

```rust
pub async fn set_cursor(
    &self,
    folder: &str,
    last_sync_at: i64,
    highest_ts: i64,
    sync_status: FolderSyncStatus,
) -> Result<(), Error>
```

This upsert operation creates a new cursor row if one doesn't exist, or updates the existing row if it does. The implementation uses SQLite's `ON CONFLICT` clause to handle both cases with a single statement:

```sql
INSERT INTO folder_cursors (folder, last_sync_at, highest_ts, sync_status)
VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(folder) DO UPDATE SET
    last_sync_at = excluded.last_sync_at,
    highest_ts   = excluded.highest_ts,
    sync_status  = excluded.sync_status
```

The sync worker calls `set_cursor` twice during a sync: once with `sync_status = Syncing` when the operation begins, and again with either `Complete` or `Failed` when it finishes. This allows the system to detect interrupted syncs on startup.

### The Freshness Signal

```rust
pub async fn latest_sync_at(&self) -> Result<Option<i64>, Error>
```

This method returns the maximum `last_sync_at` value across all folder cursors. It answers the question: "When was the most recent sync activity on *any* folder?" The return type is `Option<i64>` — `None` if no folder has ever completed a sync, otherwise the timestamp of the most recently synced folder.

The freshness signal is critical for read operations that need to present data age to the user or make caching decisions. Unlike a global cursor, this value remains accurate even when folders are synchronized independently.

## Why Independent Folder Syncs Matter

The per-folder design directly enables the CLI's `--folder` flag, which allows users to synchronize a single folder without touching the others:

```bash
imsg sync --folder inbox      # Only sync inbox
imsg sync --folder sent       # Only sync sent messages
imsg sync                    # Sync all folders
```

Each invocation updates only the relevant cursor row. The `latest_sync_at` method aggregates across all folders, so a full sync can still determine whether any folder needs attention.

This isolation also provides failure isolation. If the sent folder sync fails partway through, the inbox and deleted folder cursors remain valid. The next sync can resume from where each folder left off rather than starting over from scratch.

## Interaction with Other Store Features

The cursor system interacts with several other parts of the store:

- **Message reads** use `latest_sync_at` to determine cache validity or to display freshness indicators in the UI.
- **The outbox** (added in the same V2 migration) tracks outgoing message state independently of cursors — cursors track *incoming* sync progress, while outbox tracks *outgoing* delivery state.
- **Contact sync** uses a separate timestamp mechanism (`contacts_synced_at` in the meta table) because PBAP synchronization operates independently of message folder synchronization.

## Design Rationale and Alternatives Considered

The per-folder cursor design was chosen over several alternatives:

- **Global cursor with per-folder dirty flags**: Would require additional state to track which folders are "dirty," adding complexity without providing meaningful benefit over independent cursors.

- **Vector clocks or version vectors**: Overkill for this use case. MAP folder synchronization is linear — each folder syncs in one direction (pulling from the device), so a simple timestamp per folder is sufficient.

- **Per-folder cursors with separate "sync in progress" table**: The inline `sync_status` field was chosen to keep the schema simple and ensure atomicity. Updating a cursor and its status happens in a single statement.

The design assumes that folder names are stable and unique within a device session. The MAP protocol uses folder paths (e.g., `"telecom/msg/inbox"`), and these are used as the primary key in the `folder_cursors` table.

## Summary

Sync cursors provide fine-grained tracking of message synchronization progress on a per-folder basis. This design supports independent folder synchronization, isolates failures to the affected folder, and provides a meaningful freshness signal through the `latest_sync_at` aggregation. The implementation is straightforward — a single table with one row per folder, updated atomically by the sync worker — but it enables significantly more robust behavior than a global cursor approach.