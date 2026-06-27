-- Extends V1 with per-message outgoing state, a durable outbox, and per-folder sync cursors.

-- NULL for received messages; non-null for all outgoing rows.
ALTER TABLE messages ADD COLUMN outgoing_status TEXT;

-- Durable record of every outgoing intent created before any phone interaction.
-- A worker drains queued rows; resolved_at is set only on terminal transitions.
CREATE TABLE outbox (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    command          TEXT    NOT NULL,
    payload          TEXT    NOT NULL,
    local_message_id INTEGER REFERENCES messages(rowid),
    status           TEXT    NOT NULL DEFAULT 'queued',
    created_at       INTEGER NOT NULL,
    attempted_at     INTEGER,
    resolved_at      INTEGER,
    error            TEXT
);
CREATE INDEX idx_outbox_status ON outbox (status);

-- Per-folder sync progress; replaces the single global last_sync_at anchor in meta.
-- A partial sync on one folder cannot corrupt the cursor of another.
CREATE TABLE folder_cursors (
    folder       TEXT    NOT NULL PRIMARY KEY,
    last_sync_at INTEGER NOT NULL,
    highest_ts   INTEGER NOT NULL,
    sync_status  TEXT    NOT NULL DEFAULT 'never'
);
