//! Row and input types for the cached-contacts domain.

use formats::phone::PhoneField;

/// A cached PBAP contact, keyed by its durable vCard `UID` — not the volatile PBAP list
/// handle, which the device re-assigns on every listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactRow {
    /// vCard `UID` property value; the store's primary key for this contact.
    pub uid: String,
    /// vCard `FN` property value, or `None` if the device omitted it.
    pub display_name: Option<String>,
    /// All cached phone numbers, each carrying the raw device-reported form and, when resolvable,
    /// its E.164 canonical form.
    pub phones: Vec<PhoneField>,
}

/// A lightweight contact identity without phone numbers, returned by
/// [`crate::Store::list_contacts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactEntryRow {
    /// vCard `UID` property value; the store's primary key for this contact.
    pub uid: String,
    /// vCard `FN` property value, or `None` if the device omitted it.
    pub display_name: Option<String>,
}

/// One contact to upsert via [`crate::Store::upsert_contacts`].
///
/// `phones` replaces the complete previously-cached set for `uid` — PBAP has no per-number
/// diff signal, so a number no longer present here is dropped from the cache.
#[derive(Debug, Clone)]
pub struct NewContact {
    /// vCard `UID` property value; the store's primary key for this contact.
    pub uid: String,
    /// vCard `FN` property value, or `None` if the device omitted it.
    pub display_name: Option<String>,
    /// Current phone numbers, each carrying the raw device-reported form and, when resolvable,
    /// its E.164 canonical form.
    pub phones: Vec<PhoneField>,
}

/// Cached PBAP phonebook identity/version watermark, stored in the `meta` table.
///
/// Lets `session::contacts::sync_contacts` skip a re-pull when the device reports unchanged
/// values, and decide whether a changed `database_id` requires wiping the cache first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PbapMeta {
    /// Hex-encoded `DatabaseIdentifier`; `None` if the device never reported one.
    pub database_id: Option<String>,
    /// Hex-encoded `PrimaryVersionCounter`; `None` if the device never reported one.
    pub primary_version: Option<String>,
    /// Hex-encoded `SecondaryVersionCounter`; `None` if the device never reported one.
    pub secondary_version: Option<String>,
}
