//! Cached PBAP contact reads and writes — keyed on the durable vCard `UID`, not the volatile
//! PBAP list handle (see [`ContactRow`]).

mod meta;
mod reads;
mod types;
mod writes;

pub use types::{ContactEntryRow, ContactRow, NewContact, PbapMeta};

#[cfg(test)]
mod tests;
