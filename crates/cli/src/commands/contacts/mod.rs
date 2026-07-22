//! `contacts` subcommand: pull, list, fetch, reverse-lookup, or sync PBAP contacts.

// `pub mod` (not `mod`) is required for `live`/`store`/`sync`: `pub(crate)` on their `run`
// functions trips `redundant_pub_crate` inside a private module, while `pub` trips
// `unreachable_pub` — same reasoning as `imsg-broker-client`'s `lib.rs`.
pub mod live;
mod render;
pub mod store;
pub mod sync;

pub(crate) use live::run;
pub(crate) use store::run as run_store;
pub(crate) use sync::run as run_sync;

use crate::cli::PathArg;

/// Operation parameters for the `contacts` subcommand.
pub(crate) struct ContactsOpts {
    /// List handles/UIDs and names only, without fetching full vCards.
    pub list: bool,
    /// Fetch a single contact: by PBAP handle live/via broker, or by cached UID once opted in.
    pub get: Option<String>,
    /// Reverse-lookup a contact by phone number.
    pub lookup: Option<String>,
    /// Refresh the local contacts cache instead of reading. Consumed by the dispatcher before
    /// `run`/`run_store` ever see it — always `false` there.
    pub sync: bool,
    /// Phonebook path to query. No effect when `sync` is set.
    pub path: PathArg,
    /// When true, skip E.164 normalisation and emit numbers as stored. No effect on `--list`.
    pub raw: bool,
    /// Maximum contacts per page; `None` shows all. No effect when `sync` is set.
    pub limit: Option<u16>,
    /// 1-indexed page number; ignored when `limit` is absent. No effect when `sync` is set.
    pub page: Option<u16>,
}
