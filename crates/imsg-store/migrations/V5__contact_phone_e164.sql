-- Canonical E.164 form for each cached phone number, alongside the raw device-reported `address`.
-- Nullable: NULL when the number could not be normalised (kept solely as raw). Existing rows are
-- left NULL and repopulated on the next contact sync — no backfill.
ALTER TABLE contact_phones ADD COLUMN address_e164 TEXT;
