-- Cached PBAP contact names, keyed on the raw vCard TEL value (unnormalized — must match
-- messages.address's format for Store::threads()'s join to resolve).
CREATE TABLE contacts (
    address      TEXT NOT NULL PRIMARY KEY,
    display_name TEXT
);
