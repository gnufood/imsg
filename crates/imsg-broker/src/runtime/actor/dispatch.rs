//! Re-exports the MAP ([`map`]) and PBAP ([`pbap`]) operation handlers, plus the
//! session-error → wire-reason mapping shared by both protocols' connect paths.

use ipc::Reason;
use session::{Disposition, SessionError};

mod map;
mod pbap;

pub(in crate::runtime::actor) use map::{
    do_backfill, do_delete, do_live_folders, do_live_get, do_live_list, do_live_mark_read,
    do_live_send, do_live_threads, do_send, do_sync,
};
pub(in crate::runtime::actor) use pbap::{
    do_contacts_get, do_contacts_list, do_contacts_lookup, do_contacts_pull_all, do_sync_contacts,
};

/// Maps a session-establishment failure to the action-oriented wire [`Reason`].
///
/// Permanent failures (auth/pairing/wrong channel) become [`Reason::ConnectionRefused`];
/// transient ones (link timeout/reset) become [`Reason::DeviceUnreachable`].
pub(in crate::runtime::actor) fn connect_reason(e: &SessionError) -> Reason {
    match session::classify(e) {
        Disposition::Permanent => Reason::ConnectionRefused,
        Disposition::Transient => Reason::DeviceUnreachable,
    }
}
