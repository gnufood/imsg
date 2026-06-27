//! Maps lean `session::live` read models to serde-only `imsg-ipc` DTOs.
//!
//! The model→DTO boundary for the live-query path: `session` owns the lean models, `imsg-ipc` owns
//! the wire DTOs, and this is the single place the two meet (mirroring how [`dispatch`] maps
//! session errors to [`Reason`]).
//!
//! [`dispatch`]: super::dispatch
//! [`Reason`]: ipc::Reason

use ipc::{BodyDto, Direction, MessageDto, ThreadDto};
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
