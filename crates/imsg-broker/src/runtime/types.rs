//! Shared runtime vocabulary: the device-actor message type, connection state, the actor handles,
//! and the connect policy. Owned by the runtime boundary and scoped to `crate::runtime` — every
//! runtime submodule reads these, but nothing here escapes the crate.

use std::time::Duration;

use futures::future::BoxFuture;
use ipc::{Reason, SessionState, WatchEvent};
use map_core::client::MapClient;
use session::SessionError;
use tokio::sync::{broadcast, mpsc, oneshot, watch};

/// On-demand factory that establishes a fresh MAP session over stream type `T`.
///
/// The actor calls this on startup and on every reconnect, so the actor — not the entry point —
/// owns the connection lifecycle. Boxed so the actor stays generic only over `T`, keeping it
/// testable against in-memory duplex streams.
pub(in crate::runtime) type Connector<T> =
    Box<dyn FnMut() -> BoxFuture<'static, Result<MapClient<T>, SessionError>> + Send>;

/// Backoff and attempt limits for establishing (and re-establishing) the MAP session.
#[derive(Clone, Copy)]
pub(in crate::runtime) struct ConnectPolicy {
    /// First inter-attempt delay; doubles each retry up to `max_backoff`.
    pub(in crate::runtime) initial_backoff: Duration,
    /// Ceiling for the inter-attempt delay.
    pub(in crate::runtime) max_backoff: Duration,
    /// Maximum connect attempts per connection phase (`>= 1`).
    pub(in crate::runtime) max_attempts: u32,
    /// Wall-clock cap on a whole connection phase, across all its attempts. `None` means no
    /// deadline — persistent (daemon) mode retries until it connects, however long the device
    /// is out of range, rather than giving up like a one-shot CLI command should.
    pub(in crate::runtime) startup_budget: Option<Duration>,
}

/// The actor's MAP-session lifecycle state, published over a `watch` channel.
///
/// The terminal [`ConnState::Failed`] carries the wire [`Reason`] so connection tasks can return a
/// precise failure to held requests. Maps to the serde-only [`SessionState`] for the `Status` wire
/// frame via [`ConnState::to_wire`].
#[derive(Clone, Debug)]
pub(in crate::runtime) enum ConnState {
    /// Establishing the session (first attempt or a reconnect attempt).
    Connecting,
    /// Session live; operations may run.
    Active,
    /// A live session dropped; backoff is running before the next attempt.
    Reconnecting,
    /// Terminal: budget exhausted or a permanent error. The broker is exiting.
    Failed(Reason),
}

impl ConnState {
    /// Projects the internal state onto the serde-only wire enum (drops the [`Reason`] payload).
    pub(in crate::runtime) const fn to_wire(&self) -> SessionState {
        match self {
            Self::Connecting => SessionState::Connecting,
            Self::Active => SessionState::Active,
            Self::Reconnecting => SessionState::Reconnecting,
            Self::Failed(_) => SessionState::Failed,
        }
    }
}

/// A message sent from a connection task to the device actor.
pub(in crate::runtime) enum DeviceOp {
    /// Drain outbox then run a full MAP folder sync.
    Sync {
        /// MAP folder path, or `None` to sync all standard folders.
        folder: Option<String>,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Push an outgoing SMS via the MAP outbox.
    Send {
        /// E.164 or local phone number.
        number: String,
        /// UTF-8 message body.
        message: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Delete a MAP message by handle and folder.
    Delete {
        /// MAP message handle (opaque string from the device).
        msg_handle: String,
        /// MAP folder name (e.g. `"inbox"`).
        folder: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Run incremental catch-up backfill without a full drain.
    Backfill { reply: oneshot::Sender<ipc::BrokerResponse> },
    /// Read a folder listing live and return message DTOs; no store write.
    LiveList {
        /// MAP folder name, or `None` for inbox.
        folder: Option<String>,
        /// Keep only unread messages.
        unread: bool,
        /// Keep only messages whose resolved address equals this value.
        from: Option<String>,
        /// Earliest message datetime as a MAP string; ignored if unparseable.
        since: Option<String>,
        /// Maximum rows after `offset`.
        limit: Option<u16>,
        /// Rows to skip from the newest-first window.
        offset: u16,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Fetch one message body live by handle and return its DTO; no store write.
    LiveGet {
        /// Opaque MAP message handle.
        handle: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Aggregate live Inbox+Sent listings into per-contact thread DTOs; no store write.
    LiveThreads { reply: oneshot::Sender<ipc::BrokerResponse> },
    /// Mark a message read on the device only (non-opted-in `get --read`); no store write.
    LiveMarkRead {
        /// Opaque MAP message handle.
        handle: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Push an outgoing SMS to the device only (non-opted-in `send`); no store write.
    LiveSend {
        /// Recipient phone number.
        number: String,
        /// Message body text.
        message: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Subscribe to inbound MAP event notifications.
    ///
    /// Starts the MNS listener on the first subscriber. Returns a
    /// [`broadcast::Receiver`] for [`WatchEvent`] frames.
    Subscribe { reply: oneshot::Sender<broadcast::Receiver<WatchEvent>> },
    /// Decrement the subscriber count; stops MNS when it reaches zero.
    Unsubscribe,
}

/// Clone-able sender handle to the device actor.
///
/// Each connection task clones this to dispatch one [`DeviceOp`].
/// Returns `Err` when the actor has shut down.
#[derive(Clone)]
pub(in crate::runtime) struct DeviceHandle {
    pub(in crate::runtime) tx: mpsc::Sender<DeviceOp>,
}

impl DeviceHandle {
    /// Sends `op` to the actor.
    ///
    /// # Errors
    ///
    /// Returns [`mpsc::error::SendError`] when the actor channel is closed.
    pub(in crate::runtime) async fn send(
        &self,
        op: DeviceOp,
    ) -> Result<(), mpsc::error::SendError<DeviceOp>> {
        self.tx.send(op).await
    }

    /// Wraps a raw op sender — test-only, for handler tests that never reach the device.
    #[cfg(test)]
    pub(in crate::runtime) const fn from_sender(tx: mpsc::Sender<DeviceOp>) -> Self {
        Self { tx }
    }
}

/// Handles to a spawned device actor: the op sender, the connection-state watch, and a shutdown
/// signal that fires once the actor exits.
pub(in crate::runtime) struct ActorHandles {
    /// Op dispatch handle.
    pub(in crate::runtime) handle: DeviceHandle,
    /// Connection-state stream; connection tasks read it to gate ops and serve `Status`.
    pub(in crate::runtime) state: watch::Receiver<ConnState>,
    /// `Some(reason)` once the actor exits; `None` while still running.
    pub(in crate::runtime) shutdown: watch::Receiver<Option<TerminalReason>>,
}

/// Why the actor's connect/serve loop exited terminally.
///
/// Distinguishes a normal stop (idle timeout, external shutdown, or — in ephemeral mode — a
/// session drop with no remaining subscribers) from a connect phase that gave up for good, so the
/// persistent daemon can exit non-zero on the latter instead of looking identical to a clean
/// `imsg daemon stop` to a process supervisor.
#[derive(Clone, Debug)]
pub(in crate::runtime) enum TerminalReason {
    /// Idle timeout, an external shutdown request, or a demand-gated exit with no subscribers.
    Requested,
    /// [`Actor::try_connect`][crate::runtime::actor] exhausted its retry budget or hit a
    /// permanent MAP error before ever reaching `Active`.
    PermanentFailure(Reason),
}
