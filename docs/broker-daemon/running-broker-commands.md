# Running Broker Commands

This page traces the execution of a broker command from the moment a CLI process connects to the broker's abstract socket through to the completion of the MAP or PBAP operation on the device. The walkthrough uses a `Send` operation as the concrete example, but the same flow applies to other operational requests like `Sync`, `Delete`, and the live query variants.

## Operation and Starting Conditions

The operation traced here is an outgoing SMS send (`BrokerRequest::Send`), initiated by a CLI process that connects to an already-running broker instance. The broker must be in the `Active` state—meaning the MAP session has been established with the device—before the operation can proceed.

The participating components are:

- **CLI process**: Initiates the request, owns the connection socket
- **Broker server** (`server::serve_loop`): Accepts connections, spawns per-connection tasks
- **Connection handler** (`handler::handle_connection`): Reads requests, gates on readiness, dispatches to actor
- **Device actor** (`actor::spawn`): Owns the MAP session, executes operations serially
- **MAP dispatch** (`dispatch::do_send`): Performs the actual OBEX/MAP operation against the device

## Phase 1: Connection Acceptance

The broker binds an abstract Unix socket on startup (see [`bind_or_exit`][server::bind_or_exit] in `server/mod.rs`). The socket name is derived from the device address, ensuring that only one broker per device can hold the socket at a time—the kernel's `EADDRINUSE` on abstract names provides single-instance election without any file-based cleanup.

When a CLI process connects, [`serve_loop`][server::serve_loop] accepts the stream and spawns a new task running [`handle_connection`][handler::handle_connection]:

```rust
// server/mod.rs, lines 148-177
async fn serve_loop(handles: ActorHandles, listener: &IpcListener, ...) {
    loop {
        let stream = tokio::select! {
            result = listener.accept() => result.context("accept error")?,
        };
        let h = handle.clone();
        let st = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &h, st, dev, readiness_wait, None).await {
                tracing::warn!("connection error: {e}");
            }
        });
    }
}
```

Each connection runs in its own task, so `Watch` subscribers never block one-shot commands. The handler receives a clone of the actor's `DeviceHandle` (for sending operations) and a `watch::Receiver<ConnState>` (for reading connection state).

## Phase 2: Request Reception and Routing

[`handle_connection`][handler::handle_connection] reads one JSON-encoded request frame from the stream using length-delimited framing. The request is deserialized into a `BrokerRequest` enum variant:

```rust
// handler.rs, lines 38-67
let frame = match framed.next().await { ... };
let req: BrokerRequest = serde_json::from_slice(&frame)?;

match req {
    BrokerRequest::Status => { /* served from state watch */ }
    BrokerRequest::Watch => handle_watch(framed, handle, shutdown).await,
    BrokerRequest::Shutdown => handle_shutdown(framed, shutdown).await,
    other => handle_one_shot(framed, handle, state, readiness_wait, other).await,
}
```

The handler distinguishes between three request categories:

1. **Status**: Answered directly from the state watch, bypassing the actor entirely. This ensures `imsg status` works even when the MAP session is down or the actor is busy.
2. **Watch**: Handled by a dedicated stream handler that subscribes to the actor's broadcast channel and forwards `WatchEvent` frames.
3. **Operational requests** (Send, Sync, Delete, ListMessages, etc.): Routed to `handle_one_shot`, which gates on readiness before dispatching to the actor.

## Phase 3: Readiness Gating

Before any operational request reaches the actor, [`handle_one_shot`][handler::handle_one_shot] waits for the MAP session to reach the `Active` state. This prevents commands from failing immediately when the device is temporarily out of range during a reconnect:

```rust
// handler.rs, lines 96-107
async fn await_ready(state: &mut watch::Receiver<ConnState>, deadline: Duration) -> Ready {
    let wait = state.wait_for(|s| matches!(s, ConnState::Active | ConnState::Failed(_)));
    match tokio::time::timeout(deadline, wait).await {
        Ok(Ok(s)) => match &*s {
            ConnState::Failed(reason) => Ready::Failed(reason.clone()),
            _ => Ready::Active,
        },
        Ok(Err(_)) => Ready::Failed(Reason::Internal("broker stopped".into())),
        Err(_) => Ready::Timeout,
    }
}
```

The `readiness_wait` duration is configured in the broker's config (typically 10 seconds). If the session doesn't reach `Active` within this window, the handler returns `BrokerResponse::Failed(Reason::NotReady)` to the CLI without ever sending an operation to the actor.

## Phase 4: Operation Dispatch to Actor

Once the session is ready, `handle_one_shot` converts the `BrokerRequest` into a `DeviceOp` and sends it over the actor's mpsc channel:

```rust
// handler.rs, lines 126-138
let (tx, rx) = oneshot::channel();
let op = match req_to_op(req, tx) { ... };
if handle.send(op).await.is_err() {
    // Actor has shut down
    return send_frame(&mut framed, &BrokerResponse::Error("broker shutting down".into())).await;
}
let resp = rx.await.unwrap_or_else(|_| ...);
send_frame(&mut framed, &resp).await
```

The `DeviceOp` enum (defined in `types/device_op.rs`) carries a oneshot reply channel alongside the operation parameters. This channel is how the actor returns the result back to the connection handler.

The actor's `DeviceHandle` is simply a clone of the `mpsc::Sender<DeviceOp>`:

```rust
// types.rs, lines 140-156
pub struct DeviceHandle {
    pub tx: mpsc::Sender<DeviceOp>,
}
```

## Phase 5: Actor Processing

The device actor runs a select loop in [`serve_active`][actor::serve_active] that waits for incoming operations, MNS events, and link-state transitions:

```rust
// actor/serve/mod.rs, lines 44-79
pub async fn serve_active(&mut self, client: &mut MapClient<T>, ...) -> ServeOutcome {
    loop {
        tokio::select! {
            maybe_link = recv_link(&mut self.link) => { ... }
            maybe_ev = recv_mns(&mut self.mns_rx) => { ... }
            maybe_op = self.rx.recv() => {
                let Some(op) = maybe_op else { return ServeOutcome::Exit };
                if matches!(self.handle_op(client, pbap, op).await, OpOutcome::SessionLost) {
                    return ServeOutcome::Dropped;
                }
            }
            () = idle_sleep(self.idle) => { ... }
        }
    }
}
```

When a `DeviceOp::Send` arrives, [`handle_op`][actor::handle_op] dispatches it to the MAP handler:

```rust
// actor/serve/mod.rs, lines 86-116
async fn handle_op(&mut self, client: &mut MapClient<T>, ...) -> OpOutcome {
    match op {
        DeviceOp::Send { number, message, reply } => Self::finish_map(
            dispatch::do_send(client, &self.store, number, message).await,
            reply,
        ),
        // ... other variants
    }
}
```

## Phase 6: MAP Operation Execution

[`do_send`][dispatch::do_send] (in `actor/dispatch/map.rs`) performs the actual MAP outbox operation:

```rust
// actor/dispatch/map.rs, lines 63-79
pub async fn do_send<T: AsyncRead + AsyncWrite + Unpin>(
    client: &mut MapClient<T>,
    store: &Store,
    number: String,
    message: String,
) -> Result<BrokerResponse> {
    let now = session::util::now_ms();
    match session::outbox::send_sms(client, store, &number, &message, now).await {
        Ok(confirmation) => Ok(BrokerResponse::Text(confirmation)),
        Err(e) => classify_err(e),
    }
}
```

The handler returns `Result<BrokerResponse>`—`Ok` for successful operations, `Err` for fatal transport errors that should trigger a reconnect. Non-fatal errors (server rejections, unknown handles) are converted to `BrokerResponse::Failed` before returning.

[`finish_map`][actor::finish_map] inspects the result and decides whether the session survived:

```rust
// actor/serve/mod.rs, lines 132-152
fn finish_map(result: Result<BrokerResponse>, reply: oneshot::Sender<BrokerResponse>) -> OpOutcome {
    match result {
        Ok(resp) => {
            let _ = reply.send(resp);
            OpOutcome::Continue
        }
        Err(e) if session::outbox::is_fatal_anyhow(&e) => {
            // Transport error — session is dead
            let _ = reply.send(BrokerResponse::Failed(Reason::DeviceUnreachable));
            OpOutcome::SessionLost
        }
        Err(e) => {
            // Non-fatal — keep serving
            let _ = reply.send(BrokerResponse::Failed(Reason::OperationFailed(format!("{e:#}"))));
            OpOutcome::Continue
        }
    }
}
```

If `OpOutcome::SessionLost` is returned, `serve_active` exits its loop with `ServeOutcome::Dropped`, triggering a reconnect if subscribers remain or if the broker is running in persistent (daemon) mode.

## Phase 7: Response Return

The actor sends the response through the oneshot channel that was attached to the `DeviceOp`. The connection handler awaits this response and writes it back to the client:

```rust
// handler.rs, lines 131-137
if handle.send(op).await.is_err() { ... }
let resp = rx.await.unwrap_or_else(|_| BrokerResponse::Error("actor dropped reply".into()));
send_frame(&mut framed, &resp).await
```

[`send_frame`][handler::send_frame] serializes the response to JSON and writes a length-delimited frame:

```rust
// handler.rs, lines 189-196
async fn send_frame<S: tokio::io::AsyncWrite + Unpin>(
    framed: &mut Framed<S, LengthDelimitedCodec>,
    resp: &BrokerResponse,
) -> Result<()> {
    let bytes = Bytes::from(serde_json::to_vec(resp)?);
    framed.send(bytes).await.context("sending response frame")
}
```

The CLI deserializes this frame into a `BrokerResponse` and presents the result to the user.

## Terminal State

On successful completion:

- The CLI has received a `BrokerResponse::Text` containing the MAP confirmation (typically the sent message's handle)
- The broker's MAP session remains `Active`, ready for subsequent commands
- The actor's operation queue is empty; the select loop continues waiting for the next request, MNS event, or idle timeout

If the MAP session dies during the operation:

- The actor returns `OpOutcome::SessionLost`
- `serve_active` returns `ServeOutcome::Dropped`
- The actor's `run` loop (in `actor/inner.rs`) initiates a reconnect if `wants_mns` returns true (subscribers present or persistent mode)
- The CLI receives `BrokerResponse::Failed(Reason::DeviceUnreachable)` through the already-awaited oneshot channel

If the readiness deadline elapses before the session reaches `Active`:

- The handler returns `BrokerResponse::Failed(Reason::NotReady)` without contacting the actor
- The MAP session remains in `Connecting` or `Reconnecting` state
- The actor continues its connect/retry loop independently

## Failure Conditions Summary

| Condition | Where Detected | Response to CLI | Actor Behavior |
|-----------|----------------|-----------------|----------------|
| Session not Active within deadline | `handler::await_ready` | `Failed(Reason::NotReady)` | Continues connecting |
| Transport error during op | `dispatch::do_send` → `finish_map` | `Failed(Reason::DeviceUnreachable)` | Exits `serve_active` with `Dropped`, reconnects |
| Non-fatal MAP error | `dispatch::do_send` | `Failed(Reason::OperationFailed)` | Continues serving |
| Actor channel closed | `handle.send(op)` | `Error("broker shutting down")` | Actor has exited |
| Actor drops reply channel | `rx.await` | `Error("actor dropped reply")` | Actor has exited mid-op |

## Relationship to Other Flows

- **Watch subscriptions** follow a similar path through `handle_connection` but are handled by `handle_watch`, which subscribes to the actor's broadcast channel instead of using a oneshot reply.
- **Daemon mode** (`serve_daemon`) uses the same handler and actor, but the accept loop is wrapped in a shutdown coordinator that converges IPC `Shutdown` and SIGTERM/SIGINT into a single drain sequence (see `shutdown/run`).
- **Live queries** (`ListMessages`, `GetMessage`, `Folders`, etc.) bypass the store entirely, returning results directly from the device. The flow is identical up to the dispatch point, where they route to `handle_live_op` instead of the store-backed handlers.