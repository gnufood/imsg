//! Maps domain values — `session::live` read models, `pbap_core` contact types, and the
//! contacts sync report — to serde-only `imsg-ipc` DTOs.
//!
//! The model→DTO boundary: each domain crate owns its own shapes and knows nothing of the wire,
//! `imsg-ipc` owns the DTOs and knows nothing of the domain, and this is the single place the two
//! meet (mirroring how [`dispatch`] maps session errors to [`Reason`]).
//!
//! [`dispatch`]: super::dispatch
//! [`Reason`]: ipc::Reason

use ipc::{
    BodyDto, CardEntryDto, ContactDto, Direction, MessageDto, PhoneDto, RefreshDto, SyncReportDto,
    ThreadDto,
};
use pbap_core::{CardEntry, Contact};
use session::contacts::{Refresh, SyncReport};
use session::live::models::{Direction as LiveDirection, LiveBody, LiveMessage, LiveThread};

pub(in crate::runtime::actor) fn to_message_dto(m: LiveMessage) -> MessageDto {
    MessageDto {
        handle: m.handle,
        timestamp_ms: m.timestamp_ms,
        address: m.address,
        folder: m.folder,
        read: m.read,
        text: m.text,
    }
}

pub(in crate::runtime::actor) const fn to_sync_report_dto(r: SyncReport) -> SyncReportDto {
    match r {
        SyncReport::UpToDate => SyncReportDto::UpToDate,
        SyncReport::Refreshed(Refresh { listed, pull_failed, no_uid, written, wiped }) => {
            SyncReportDto::Refreshed(RefreshDto { listed, pull_failed, no_uid, written, wiped })
        }
    }
}

pub(in crate::runtime::actor) fn to_thread_dto(t: LiveThread) -> ThreadDto {
    ThreadDto { address: t.address, latest_ms: t.latest_ms, total: t.total, unread: t.unread }
}

pub(in crate::runtime::actor) fn to_body_dto(b: LiveBody) -> BodyDto {
    BodyDto {
        handle: b.handle,
        direction: to_direction(b.direction),
        address: b.address,
        folder: b.folder,
        read: b.read,
        text: b.text,
    }
}

const fn to_direction(d: LiveDirection) -> Direction {
    match d {
        LiveDirection::Received => Direction::Received,
        LiveDirection::Sent => Direction::Sent,
    }
}

pub(in crate::runtime::actor) fn to_card_entry_dto(e: &CardEntry) -> CardEntryDto {
    CardEntryDto { handle: e.handle().to_owned(), name: e.name().map(str::to_owned) }
}

pub(in crate::runtime::actor) fn to_contact_dto(c: Contact) -> ContactDto {
    let phones = c
        .phones()
        .iter()
        .map(|p| PhoneDto { raw: p.raw().to_owned(), e164: p.e164().map(str::to_owned) })
        .collect();
    ContactDto { display_name: c.display_name, uid: c.uid, phones }
}
