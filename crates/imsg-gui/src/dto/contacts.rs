//! Cached-contact DTOs for the GUI's local-store read path.
//!
//! Distinct from `imsg-ipc`'s `ContactDto`/`CardEntryDto` (the broker wire contract for live PBAP
//! ops) and from `imsg-store`'s own row types, for the same reason as `dto::MessageDto`/
//! `ThreadDto`: the store's row shape can change without forcing a frontend/TS-binding change.

use serde::Serialize;

/// A lightweight contact identity (no phone numbers), for list views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct ContactEntryDto {
    /// The cached contact's vCard `UID` — stable across PBAP syncs, unlike a PBAP list handle.
    pub uid: String,
    /// vCard `FN` property value, or `None` if the device omitted it.
    pub display_name: Option<String>,
}

impl From<&store::ContactEntryRow> for ContactEntryDto {
    fn from(row: &store::ContactEntryRow) -> Self {
        Self { uid: row.uid.clone(), display_name: row.display_name.clone() }
    }
}

/// A full cached contact, including every phone number on file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct ContactDto {
    /// The cached contact's vCard `UID` — stable across PBAP syncs, unlike a PBAP list handle.
    pub uid: String,
    /// vCard `FN` property value, or `None` if the device omitted it.
    pub display_name: Option<String>,
    /// Phone numbers exactly as the device's vCard reported them.
    pub phones: Vec<String>,
}

impl From<&store::ContactRow> for ContactDto {
    fn from(row: &store::ContactRow) -> Self {
        Self {
            uid: row.uid.clone(),
            display_name: row.display_name.clone(),
            phones: row.phones.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
