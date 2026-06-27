CREATE TABLE messages (
    rowid        INTEGER PRIMARY KEY AUTOINCREMENT,
    map_handle   TEXT    NOT NULL UNIQUE,
    timestamp_ms INTEGER NOT NULL,
    folder       TEXT    NOT NULL DEFAULT '',
    direction    INTEGER NOT NULL CHECK (direction IN (0, 1)),
    address      TEXT    NOT NULL DEFAULT '',
    status       INTEGER NOT NULL DEFAULT 0,
    synced_at    INTEGER NOT NULL DEFAULT 0,
    text         TEXT    NOT NULL DEFAULT ''
);
CREATE INDEX idx_messages_timestamp    ON messages (timestamp_ms);
CREATE INDEX idx_messages_folder_time  ON messages (folder, timestamp_ms);
CREATE INDEX idx_messages_address_time ON messages (address, timestamp_ms);

CREATE TABLE meta (
    key   TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);
