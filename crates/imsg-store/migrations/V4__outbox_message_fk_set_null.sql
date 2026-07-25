-- V2's outbox.local_message_id FK has no ON DELETE action, so RESTRICT (SQLite's default)
-- blocks deleting any message that ever passed through the outbox (e.g. imsg delete on a sent
-- message) with a raw FOREIGN KEY constraint failed. SET NULL: nothing joins through
-- local_message_id to reconstruct the message (it's a plain field on OutboxRow), so severing
-- the link loses no functionality, and the outbox row itself — command/payload/status/timestamps
-- — is preserved as the durable audit trail it's meant to be.
--
-- SQLite has no ALTER TABLE support for foreign key clauses, so this rebuilds the table.
CREATE TABLE outbox_new (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    command          TEXT    NOT NULL,
    payload          TEXT    NOT NULL,
    local_message_id INTEGER REFERENCES messages(rowid) ON DELETE SET NULL,
    status           TEXT    NOT NULL DEFAULT 'queued',
    created_at       INTEGER NOT NULL,
    attempted_at     INTEGER,
    resolved_at      INTEGER,
    error            TEXT
);
INSERT INTO outbox_new
    SELECT id, command, payload, local_message_id, status, created_at, attempted_at, resolved_at, error
    FROM outbox;
DROP TABLE outbox;
ALTER TABLE outbox_new RENAME TO outbox;
CREATE INDEX idx_outbox_status ON outbox (status);
