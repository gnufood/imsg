# Runtime Types Reference

This reference documents the internal types that form the broker's runtime vocabulary. These types are scoped to `crate::runtime` and are not exposed outside the crate.

## Device Operations

### `DeviceOp`

A message sent from a connection task to the device actor. All operations are serialised through the actor, which owns the MAP and PBAP session lifecycle.

```rust
pub(in crate::runtime) enum DeviceOp {
    Sync { folder: Option<String>, reply: oneshot::Sender<ipc::BrokerResponse> },
    Send { number: String, message: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    Delete { msg_handle: String, folder: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    Backfill { reply: oneshot::Sender<ipc::BrokerResponse> },
    LiveList { folder: Option<String>, unread: bool, from: Option<String>, since: Option<String>, limit: Option<u16>, offset: u16, reply: oneshot::Sender<ipc::BrokerResponse> },
    LiveGet { handle: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    LiveThreads { reply: oneshot::Sender<ipc::BrokerResponse> },
    LiveFolders { reply: oneshot::Sender<ipc::BrokerResponse> },
    LiveMarkRead { handle: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    LiveSend { number: String, message: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    SyncContacts { reply: oneshot::Sender<ipc::BrokerResponse> },
    ListContacts { path: Option<String>, limit: Option<u16>, offset: u16, reply: oneshot::Sender<ipc::BrokerResponse> },
    GetContact { path: Option<String>, handle: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    LookupContact { path: Option<String>, number: String, reply: oneshot::Sender<ipc::BrokerResponse> },
    PullAllContacts { path: Option<String>, limit: Option<u16>, offset: u16, reply: oneshot::Sender<ipc::BrokerResponse> },
    Subscribe { reply: oneshot::Sender<broadcast::Receiver<WatchEvent>> },
    Unsubscribe,
}
```

#### Operation Categories

| Category | Operations | Session Used |
|----------|------------|--------------|
| Store-backed sync | `Sync`, `Send`, `Delete`, `Backfill`, `SyncContacts` | MAP (messages) or PBAP (contacts) |
| Live read (no store) | `LiveList`, `LiveGet`, `LiveThreads`, `LiveFolders`, `LiveMarkRead`, `LiveSend`, `ListContacts`, `GetContact`, `LookupContact`, `PullAllContacts` | MAP or PBAP |
| Event subscription | `Subscribe`, `Unsubscribe` | MAP (MNS listener) |

#### MAP Operations

- **Sync**: Drains the outbox, then runs a full MAP folder sync. The `folder` parameter specifies a MAP folder path, or `None` for all standard folders.
- **Send**: Pushes an outgoing SMS via the MAP outbox. Takes an E.164 or local phone number and UTF-8 message body.
- **Delete**: Deletes a MAP message by handle and folder. The `msg_handle` is an opaque string from the device; `folder` is the MAP folder name (e.g., `"inbox"`).
- **Backfill**: Runs incremental catch-up backfill without a full drain.
- **LiveList**: Reads a folder listing live and returns message DTOs without writing to the store. Supports filtering by `unread`, `from` (resolved address), `since` (earliest message datetime as MAP string), `limit`, and `offset`.
- **LiveGet**: Fetches one message body live by handle, returns a DTO without writing to the store.
- **LiveThreads**: Aggregates live Inbox+Sent listings into per-contact thread DTOs.
- **LiveFolders**: Lists the device's MAP message folders under `telecom/msg`.
- **LiveMarkRead**: Marks a message read on the device only (non-opted-in `get --read`).
- **LiveSend**: Pushes an outgoing SMS to the device only (non-opted-in `send`).
- **Subscribe**: Subscribes to inbound MAP event notifications. Starts the MNS listener on the first subscriber. Returns a `broadcast::Receiver` for `WatchEvent` frames.
- **Unsubscribe**: Decrements the subscriber count; stops MNS when it reaches zero.

#### PBAP Operations

- **SyncContacts**: Pulls the main phonebook and upserts contact display names into the local contacts cache. Runs against the actor's held PBAP session, independent of the MAP session.
- **ListContacts**: Lists a phonebook live and returns entry DTOs without writing to the store. The `path` parameter is a lowercase PBAP phonebook path name, or `None` for the main phonebook.
- **GetContact**: Fetches one contact vCard live by handle.
- **LookupContact**: Reverse-look up a contact by phone number live, then pull its vCard.
- **PullAllContacts**: Pulls every contact vCard in a phonebook live.

## Connection State

### `ConnState`

The actor's MAP-session lifecycle state, published over a `watch` channel.

```rust
pub(in crate::runtime) enum ConnState {
    Connecting,
    Active,
    Reconnecting,
    Failed(Reason),
}
```

| State | Meaning |
|-------|---------|
| `Connecting` | Establishing the session (first attempt or a reconnect attempt). |
| `Active` | Session is live; operations may run. |
| `Reconnecting` | A live session dropped; backoff is running before the next attempt. |
| `Failed(Reason)` | Terminal: budget exhausted or a permanent error. The broker is exiting. The `Reason` carries the wire failure code. |

#### Wire Projection

The `to_wire` method projects the internal state onto the serde-only wire enum, dropping the `Reason` payload:

```rust
impl ConnState {
    pub(in crate::runtime) const fn to_wire(&self) -> SessionState {
        match self {
            Self::Connecting => SessionState::Connecting,
            Self::Active => SessionState::Active,
            Self::Reconnecting => SessionState::Reconnecting,
            Self::Failed(_) => SessionState::Failed,
        }
    }
}
```

### `LinkState`

Physical reachability of the device, as reported by the transport itself. Independent of MAP traffic.

```rust
pub(in crate::runtime) enum LinkState {
    Up,
    Down,
}
```

- **Up**: The transport reports the device reachable.
- **Down**: The transport reports the device unreachable. Carries no reason — the actor treats any `Down` the same way it treats a session that died mid-op.

## Connection Policy

### `ConnectPolicy`

Backoff and attempt limits for establishing (and re-establishing) the MAP session.

```rust
pub(in crate::runtime) struct ConnectPolicy {
    pub(in crate::runtime) initial_backoff: Duration,
    pub(in crate::runtime) max_backoff: Duration,
    pub(in crate::runtime) max_attempts: u32,
    pub(in crate::runtime) startup_budget: Option<Duration>,
}
```

| Field | Description |
|-------|-------------|
| `initial_backoff` | First inter-attempt delay; doubles each retry up to `max_backoff`. |
| `max_backoff` | Ceiling for the inter-attempt delay. |
| `max_attempts` | Maximum connect attempts per connection phase (`>= 1`). |
| `startup_budget` | Wall-clock cap on a whole connection phase, across all its attempts. `None` means no deadline — persistent (daemon) mode retries until it connects. |

#### Policy Construction

Two policy builders exist:

- **`build_policy`**: For one-shot (`serve`) mode — bounded attempts within a wall-clock budget, so a CLI command fails fast.
- **`build_daemon_policy`**: For persistent (`serve_daemon`) mode — unbounded attempts, no wall-clock deadline.

## Connector Factories

### `Connector<T>`

On-demand factory that establishes a fresh MAP session over stream type `T`.

```rust
pub(in crate::runtime) type Connector<T> =
    Box<dyn FnMut() -> BoxFuture<'static, Result<MapClient<T>, SessionError>> + Send>;
```

The actor calls this on startup and on every reconnect. The actor — not the entry point — owns the connection lifecycle. Boxed so the actor stays generic only over `T`, keeping it testable against in-memory duplex streams.

### `PbapConnector<T>`

On-demand factory that establishes a PBAP session over stream type `T`.

```rust
pub(in crate::runtime) type PbapConnector<T> =
    Box<dyn FnMut() -> BoxFuture<'static, Result<PbapClient<T>, SessionError>> + Send>;
```

Same contract as `Connector`: the actor calls this lazily on the first PBAP-touching `DeviceOp` and again on reconnect after that session drops. Unlike the MAP session, PBAP connects lazily (there is no PBAP-side push notification requiring it up front) and a lost PBAP session never affects the MAP session's liveness, or vice versa — the two are independent fault domains.

### `LinkWatcher`

On-demand factory that subscribes to the device's link-state transitions.

```rust
pub(in crate::runtime) type LinkWatcher = Box<dyn FnMut() -> BoxFuture<'static, LinkEvents> + Send>;
```

Same contract as `Connector`: called by the actor on first use and again after a stream ends. A transport with no notion of link liveness supplies a stream that never yields.

### `LinkEvents`

A stream of `LinkState` transitions for one device.

```rust
pub(in crate::runtime) type LinkEvents = Pin<Box<dyn Stream<Item = LinkState> + Send>>;
```

Termination means the transport can no longer report on the link — never that the link is healthy. The actor resubscribes via its `LinkWatcher` rather than treating the end of the stream as silence.

### `Connectors<T>`

The MAP and PBAP connectors, bundled — every caller constructs and passes both together.

```rust
pub(in crate::runtime) struct Connectors<T> {
    pub(in crate::runtime) map: Connector<T>,
    pub(in crate::runtime) pbap: PbapConnector<T>,
    pub(in crate::runtime) link: LinkWatcher,
}
```

| Field | Description |
|-------|-------------|
| `map` | Establishes the persistent MAP session the actor owns for its whole lifetime. |
| `pbap` | Establishes the persistent PBAP session, connected lazily on first use. |
| `link` | Subscribes to the device's link-state reports, so a drop is noticed without traffic. |

## Actor Handles

### `DeviceHandle`

Clone-able sender handle to the device actor. Each connection task clones this to dispatch one `DeviceOp`.

```rust
pub(in crate::runtime) struct DeviceHandle {
    pub(in crate::runtime) tx: mpsc::Sender<DeviceOp>,
}
```

#### `send`

```rust
impl DeviceHandle {
    pub(in crate::runtime) async fn send(
        &self,
        op: DeviceOp,
    ) -> Result<(), mpsc::error::SendError<DeviceOp>>
}
```

Sends `op` to the actor. Returns `Err` when the actor has shut down.

### `ActorHandles`

Handles to a spawned device actor: the op sender, the connection-state watch, and a shutdown signal that fires once the actor exits.

```rust
pub(in crate::runtime) struct ActorHandles {
    pub(in crate::runtime) handle: DeviceHandle,
    pub(in crate::runtime) state: watch::Receiver<ConnState>,
    pub(in crate::runtime) shutdown: watch::Receiver<Option<TerminalReason>>,
}
```

| Field | Description |
|-------|-------------|
| `handle` | Op dispatch handle. |
| `state` | Connection-state stream; connection tasks read it to gate ops and serve `Status`. |
| `shutdown` | `Some(reason)` once the actor exits; `None` while still running. |

### `TerminalReason`

Why the actor's connect/serve loop exited terminally.

```rust
pub(in crate::runtime) enum TerminalReason {
    Requested,
    PermanentFailure(Reason),
}
```

| Variant | Description |
|---------|-------------|
| `Requested` | Idle timeout, an external shutdown request, or a demand-gated exit with no subscribers. |
| `PermanentFailure(Reason)` | `Actor::try_connect` exhausted its retry budget or hit a permanent MAP error before ever reaching `Active`. |

Distinguishes a normal stop from a connect phase that gave up for good, so the persistent daemon can exit non-zero on the latter instead of looking identical to a clean `imsg daemon stop` to a process supervisor.

## Test Utilities

### `no_link_events`

A `LinkWatcher` that never reports a transition, leaving the actor to learn of a dead session mid-op.

```rust
#[cfg(test)]
pub(in crate::runtime) fn no_link_events() -> LinkWatcher
```

Test-only: every production path is BlueZ-backed. A transport with no link-liveness notion of its own would want this shape.

## See Also

- [Server Reference](./broker-daemon-overview.md#abstract-socket-binding-and-single-instance-election) — Socket binding, accept loop, and connection handling
- [Shutdown Reference](./daemon-graceful-shutdown.md) — Graceful shutdown coordinator for persistent mode
- [Handler Reference](./running-broker-commands.md#phase-3-readiness-gating) — Per-connection request handling and readiness gating