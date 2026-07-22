//! MNS (Message Notification Service) event handling — store write, wire mapping, and the
//! subscriber-count policy for whether the MNS listener should be running at all. Split out of
//! `super` to keep that module under the size ceiling.

use ipc::{EventType, WatchEvent};
use map_core::client::MapClient;
use store::Store;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, mpsc};

use super::super::OpOutcome;

/// Awaits the next MNS event, or `pending` when MNS is not running.
pub(in crate::runtime::actor::serve) async fn recv_mns(
    rx: &mut Option<mpsc::Receiver<session::MnsEvent>>,
) -> Option<session::MnsEvent> {
    match rx {
        Some(r) => r.recv().await,
        None => std::future::pending().await,
    }
}

/// Maps `session::EventType` (`map_core`) to the wire [`EventType`] — exhaustive, no wildcard
/// arm, so a new `session::EventType` variant fails to compile here instead of silently
/// dropping through as an unmapped event.
const fn to_wire_event_type(ev: session::EventType) -> EventType {
    match ev {
        session::EventType::NewMessage => EventType::NewMessage,
        session::EventType::DeliverySuccess => EventType::DeliverySuccess,
        session::EventType::SendingSuccess => EventType::SendingSuccess,
        session::EventType::DeliveryFailure => EventType::DeliveryFailure,
        session::EventType::SendingFailure => EventType::SendingFailure,
        session::EventType::MessageDeleted => EventType::MessageDeleted,
        session::EventType::MessageShift => EventType::MessageShift,
        session::EventType::MemoryFull => EventType::MemoryFull,
        session::EventType::MemoryAvailable => EventType::MemoryAvailable,
        session::EventType::ReadStatusChanged => EventType::ReadStatusChanged,
    }
}

/// Flattens an [`session::MnsEvent`] into the wire [`WatchEvent`].
fn mns_to_watch(ev: &session::MnsEvent) -> WatchEvent {
    WatchEvent {
        event_type: to_wire_event_type(ev.event_type()),
        handle: ev.handle().map(str::to_owned),
        folder: ev.folder().map(str::to_owned),
        old_folder: ev.old_folder().map(str::to_owned),
        msg_type: ev.msg_type().map(str::to_owned),
        datetime: ev.datetime().map(str::to_owned),
    }
}

/// Writes an MNS event to the store, then fans it out to `Watch` subscribers regardless of the
/// write outcome — subscribers should still see the raw event even if persistence failed.
///
/// Returns [`OpOutcome::SessionLost`] when the MAP fetch behind a `NewMessage` event hits a fatal
/// transport error — the caller should reconnect. Non-fatal store/MAP errors are logged and
/// otherwise ignored; the event is not retried (`Store::reconcile_outgoing` and periodic backfill
/// are the correctness fallback for anything missed).
pub(in crate::runtime::actor::serve) async fn on_mns_event<T: AsyncRead + AsyncWrite + Unpin>(
    ev: &session::MnsEvent,
    client: &mut MapClient<T>,
    store: &Store,
    watch_tx: &broadcast::Sender<WatchEvent>,
) -> OpOutcome {
    let now = session::util::now_ms();
    let mut outcome = OpOutcome::Continue;
    if let Err(e) = session::watch::handle_mns_event(ev, client, store, now).await {
        if session::outbox::is_fatal_anyhow(&e) {
            tracing::warn!("MNS event handling lost the session: {e:#}");
            outcome = OpOutcome::SessionLost;
        } else {
            tracing::warn!("MNS event store write failed: {e:#}");
        }
    }
    let _ = watch_tx.send(mns_to_watch(ev));
    outcome
}

/// True when MNS should be running: a live `Watch` subscriber, or persistent (daemon) mode,
/// signaled by `idle == None` (the mode switch introduced for the idle timeout).
pub(in crate::runtime::actor) const fn wants_mns(
    watch_count: u32,
    idle: Option<std::time::Duration>,
) -> bool {
    watch_count > 0 || idle.is_none()
}

#[cfg(test)]
mod tests;
