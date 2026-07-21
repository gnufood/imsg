-- Cached PBAP contacts, keyed on the durable vCard UID — not the volatile PBAP list handle,
-- which the device re-assigns on every listing (see formats::vcard::Contact::uid).
CREATE TABLE contacts (
    uid          TEXT NOT NULL PRIMARY KEY,
    display_name TEXT
);

-- One row per cached phone number, owned by exactly one contact — `address` is the PK so a
-- number can't fan out across two contacts and corrupt Store::threads's aggregate join.
-- `address` is stored exactly as the device's vCard reports it (unnormalized) — must match
-- messages.address's format for that join to resolve. ON DELETE CASCADE requires
-- PRAGMA foreign_keys = ON, set at connection open.
CREATE TABLE contact_phones (
    address TEXT NOT NULL PRIMARY KEY,
    uid     TEXT NOT NULL REFERENCES contacts(uid) ON DELETE CASCADE
);
CREATE INDEX idx_contact_phones_uid ON contact_phones (uid);
