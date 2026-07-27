-- Canonical E.164 form for each message address, alongside the raw device-reported `address`.
-- Nullable: NULL when the number could not be normalised (kept solely as raw). Existing rows are
-- left NULL and repopulated on the next sync — no backfill. Reads coalesce to canonical.
ALTER TABLE messages ADD COLUMN address_e164 TEXT;
