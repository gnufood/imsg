//! Shared runtime vocabulary: the device-actor message type ([`DeviceOp`], in [`device_op`]),
//! connection state, the actor handles, and the connect policy. Owned by the runtime boundary
//! and scoped to `crate::runtime` — every runtime submodule reads these, but nothing here
//! escapes the crate.

use std::pin::Pin;
use std::time::Duration;

use futures::future::BoxFuture;
use futures::Stream;
use ipc::{Reason, SessionState};
use map_core::client::MapClient;
use pbap_core::client::PbapClient;
use session::SessionError;
use tokio::sync::{mpsc, watch};

mod device_op;

pub(in crate::runtime) use device_op::DeviceOp;

/// On-demand factory that establishes a fresh MAP session over stream type `T`.
///
/// The actor calls this on startup and on every reconnect, so the actor — not the entry point —
/// owns the connection lifecycle. Boxed so the actor stays generic only over `T`, keeping it
/// testable against in-memory duplex streams.
pub(in crate::runtime) type Connector<T> =
    Box<dyn FnMut() -> BoxFuture<'static, Result<MapClient<T>, SessionError>> + Send>;

/// On-demand factory that establishes a PBAP session over stream type `T`.
///
/// Same contract as [`Connector`]: the actor calls this lazily on the first PBAP-touching
/// [`DeviceOp`] and again on reconnect after that session drops, holding the client in between —
/// it is not a fresh connect per op. Unlike the MAP session, PBAP connects lazily (there is no
/// PBAP-side push notification requiring it up front) and a lost PBAP session never affects the
/// MAP session's liveness, or vice versa; the two are independent fault domains sharing the same
/// persistent-session shape.
pub(in crate::runtime) type PbapConnector<T> =
    Box<dyn FnMut() -> BoxFuture<'static, Result<PbapClient<T>, SessionError>> + Send>;

/// Physical reachability of the device, as reported by the transport itself.
///
/// Independent of MAP traffic: the transport knows the link is gone even when the actor is idle
/// and no operation has failed. Carries no reason — the actor treats any `Down` the same way it
/// treats a session that died mid-op.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum LinkState {
    /// The transport reports the device reachable.
    Up,
    /// The transport reports the device unreachable.
    Down,
}

/// A stream of [`LinkState`] transitions for one device.
///
/// Termination means the transport can no longer report on the link — never that the link is
/// healthy. The actor resubscribes via its [`LinkWatcher`] rather than treating the end of the
/// stream as silence.
pub(in crate::runtime) type LinkEvents = Pin<Box<dyn Stream<Item = LinkState> + Send>>;

/// On-demand factory that subscribes to the device's link-state transitions.
///
/// Same contract as [`Connector`]: called by the actor on first use and again after a stream
/// ends, so the actor owns the subscription lifecycle. A transport with no notion of link
/// liveness supplies a stream that never yields.
pub(in crate::runtime) type LinkWatcher = Box<dyn FnMut() -> BoxFuture<'static, LinkEvents> + Send>;

/// A [`LinkWatcher`] that never reports a transition, leaving the actor to learn of a dead
/// session mid-op as it did before link watching existed.
///
/// Test-only: every production path is BlueZ-backed. A transport with no link-liveness notion
/// of its own (the deferred hub/spoke path) would want this shape.
#[cfg(test)]
pub(in crate::runtime) fn no_link_events() -> LinkWatcher {
    Box::new(|| {
        Box::pin(async {
            let events: LinkEvents = Box::pin(futures::stream::pending());
            events
        })
    })
}

/// The MAP and PBAP connectors, bundled — every caller constructs and passes both together.
pub(in crate::runtime) struct Connectors<T> {
    /// Establishes the persistent MAP session the actor owns for its whole lifetime.
    pub(in crate::runtime) map: Connector<T>,
    /// Establishes the persistent PBAP session, connected lazily on first use.
    pub(in crate::runtime) pbap: PbapConnector<T>,
    /// Subscribes to the device's link-state reports, so a drop is noticed without traffic.
    pub(in crate::runtime) link: LinkWatcher,
}

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
