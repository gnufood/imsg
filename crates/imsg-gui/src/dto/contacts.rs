//! Cached-contact DTOs for the GUI's local-store read path.
//!
//! Distinct from `imsg-ipc`'s `ContactDto`/`CardEntryDto` (the broker wire contract for live PBAP
//! ops) and from `imsg-store`'s own row types, for the same reason as `dto::MessageDto`/
//! `ThreadDto`: the store's row shape can change without forcing a frontend/TS-binding change.

use serde::Serialize;

/// One cached phone number in both its raw device-reported form and its canonical E.164 form.
///
/// `e164` is `None` when the number could not be normalised. Raw is always present so the
/// frontend can show the number as the device reported it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
pub struct PhoneDto {
    /// The number exactly as the device's vCard reported it (whitespace-stripped).
    pub raw: String,
    /// The canonical E.164 form, or `None` if the number could not be resolved.
    pub e164: Option<String>,
}

impl From<&store::PhoneField> for PhoneDto {
    fn from(p: &store::PhoneField) -> Self {
        Self { raw: p.raw().to_owned(), e164: p.e164().map(str::to_owned) }
    }
}

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
    /// Phone numbers in vCard order, each carrying raw and canonical forms.
    pub phones: Vec<PhoneDto>,
}

impl From<&store::ContactRow> for ContactDto {
    fn from(row: &store::ContactRow) -> Self {
        Self {
            uid: row.uid.clone(),
            display_name: row.display_name.clone(),
            phones: row.phones.iter().map(PhoneDto::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests;
