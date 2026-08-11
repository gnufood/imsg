# Daemon Graceful Shutdown

A persistent daemon must handle shutdown requests from multiple sources—IPC clients sending `Shutdown`, system signals like SIGTERM and SIGINT—and converge them into a single, bounded drain sequence. This page explains how the shutdown coordinator achieves this, why the design choices matter, and what guarantees it provides to both operators and clients.

## The shutdown problem

In daemon mode, the broker runs indefinitely with no idle timeout. Unlike the ephemeral one-shot broker, which exits when all subscribers disconnect or after a fixed idle period, the persistent daemon needs explicit termination. But termination isn't instantaneous: there may be in-flight operations, active connections, and a device actor that holds a live MAP session over Bluetooth.

The challenge is threefold:

1. **Multiple triggers**: An IPC client may request shutdown via the `BrokerRequest::Shutdown` message, or the process may receive a Unix signal (SIGTERM from a service manager, SIGINT from Ctrl+C).
2. **In-flight work**: Connections that have already been accepted must be allowed to complete rather than being aborted mid-operation.
3. **Bounded convergence**: The process must eventually exit—never hang forever waiting for a connection that never finishes, or a Bluetooth session that won't go away.

## Converging on one cancellation token

The coordinator uses a single `CancellationToken` from `tokio_util::sync` as the convergence point. Both the IPC `Shutdown` handler and the signal handlers cancel this token:

```rust
// From shutdown.rs — the token is shared by all shutdown sources
let token = CancellationToken::new();
#[cfg(unix)]
install_signal_handlers(token.clone());
```

When a client sends `BrokerRequest::Shutdown`, the handler cancels the token:

```rust
// From handler.rs — the Shutdown request cancels the coordinator
async fn handle_shutdown<S: tokio::io::AsyncWrite + Unpin>(
    mut framed: Framed<S, LengthDelimitedCodec>,
    shutdown: Option<&CancellationToken>,
) -> Result<()> {
    let Some(token) = shutdown else {
        return send_frame(&mut framed, &BrokerResponse::Error(
            "shutdown not supported by an ephemeral broker".into()
        )).await;
    };
    token.cancel();
    send_frame(&mut framed, &BrokerResponse::Ok).await
}
```

Similarly, the signal handlers cancel the token on SIGTERM or SIGINT. This design means any shutdown trigger—IPC or signal—follows exactly the same code path, eliminating divergent behavior.

## The drain sequence

Once the token is cancelled, the accept loop exits and a carefully ordered drain sequence begins. The sequence is bounded by two `DRAIN_TIMEOUT` constants (currently five seconds each):

1. **Stop accepting**: The accept loop exits immediately when the token is cancelled. No new connections are accepted.

2. **Drop the listener immediately**: This is a deliberate design choice. The listener is dropped *before* waiting for in-flight connections to finish, not after. The comment in the code explains why:

   > Drop the listener now, not when this function returns — a connection that manages to `connect()` into the kernel backlog after we stop accepting but before this drop would otherwise sit unread until the listener eventually goes away, then see a raw connection reset instead of a clean, immediate refusal.

   If the listener remained open during the entire drain window, a client connecting during that window would queue in the kernel's accept backlog. Since the accept loop has exited, no one would ever call `accept()` on that connection. The client would eventually time out or receive a raw connection reset. By dropping the listener immediately, the kernel sends an immediate "connection refused" (or equivalent) to any client that races the shutdown.

3. **Wait for connection tasks**: Each accepted connection runs in a task tracked by a `TaskTracker`. The coordinator waits for all tracked tasks to complete:

   ```rust
   tasks.close();
   let _ = tokio::time::timeout(DRAIN_TIMEOUT, tasks.wait()).await;
   ```

   This ensures that in-flight operations (Sync, Send, Delete, etc.) complete normally rather than being cancelled mid-flight.

4. **Drop the actor handle**: The `DeviceHandle` (an `mpsc::Sender<DeviceOp>`) is dropped. This is the actor's only mechanism to detect that the broker is shutting down—once all handles are dropped and the channel empties, the actor knows to exit.

5. **Wait for actor exit**: The coordinator waits for the actor to publish its `TerminalReason` on the shutdown watch channel:

   ```rust
   let _ = tokio::time::timeout(DRAIN_TIMEOUT, shutdown.wait_for(Option::is_some)).await;
   ```

   The actor publishes a reason (either `Requested` for a clean shutdown or `PermanentFailure` for an unrecoverable error) when it exits.

6. **Surface the outcome**: If the actor exited with a `PermanentFailure`, the coordinator returns an error so the process exits non-zero. This allows a service supervisor with `RestartPolicy::OnFailure` to react to genuine failures rather than treating them as identical to a clean stop.

## Why bounded timeouts matter

The five-second bounds on connection completion and actor exit are not arbitrary—they prevent the daemon from hanging indefinitely. Consider what happens without bounds:

- A client could start a large file transfer and never finish.
- The Bluetooth link could become unresponsive, causing the MAP session to hang.
- A misbehaving actor could deadlock.

With bounded timeouts, the worst-case shutdown time is deterministic: at most ten seconds (two timeouts) plus the time to close the listener and drop handles. This is acceptable for a service manager that needs reliable restarts.

## Distinguishing clean stops from failures

A critical design detail is that the coordinator distinguishes between a clean shutdown request and a permanent connection failure:

```rust
// From shutdown.rs — permanent failures become errors
match reason {
    Some(TerminalReason::PermanentFailure(reason)) => {
        Err(anyhow::anyhow!("daemon stopped: unrecoverable MAP failure: {reason:?}"))
    }
    _ => Ok(()),
}
```

This matters for service supervisors. If the daemon exits with code 0 for both a clean `imsg daemon stop` and an unrecoverable Bluetooth failure, a supervisor with `RestartPolicy::OnFailure` would never restart on the latter—treating a fatal error as success. By returning an error on `PermanentFailure`, the daemon ensures the supervisor can distinguish the two cases and react appropriately.

The actor publishes `TerminalReason::PermanentFailure` when its connect phase exhausts its retry budget or hits a permanent MAP error (such as authentication failure or an unsupported device), as opposed to `TerminalReason::Requested` which covers idle timeout, external shutdown, or a demand-gated exit with no subscribers.

## Contrast with ephemeral mode

The ephemeral one-shot broker (used by CLI commands) does not use the shutdown coordinator. Its accept loop runs through `serve_loop` in `server/mod.rs`, which exits when the actor signals shutdown but has no mechanism for IPC shutdown requests or signal handling. This is intentional:

- CLI commands are short-lived; they exit when done.
- Adding graceful shutdown to the ephemeral path would introduce complexity and risk to the common case.
- The daemon path explicitly opts into the coordinator via `serve_daemon`, which calls `shutdown::run` instead of `serve_loop`.

The handler code reflects this distinction: it accepts a `shutdown: Option<&CancellationToken>` parameter, and only daemon-mode handlers have a token to pass:

```rust
// From handler.rs — the shutdown parameter is None for ephemeral mode
pub(in crate::runtime) async fn handle_connection<S>(
    stream: S,
    handle: &DeviceHandle,
    state: watch::Receiver<ConnState>,
    device: String,
    readiness_wait: Duration,
    shutdown: Option<&CancellationToken>,  // None for ephemeral
) -> Result<()>
```

## Summary

The graceful shutdown coordinator provides three key guarantees:

1. **Convergence**: IPC `Shutdown` requests, SIGTERM, and SIGINT all trigger the same drain sequence via a shared `CancellationToken`.

2. **Bounded completion**: The drain sequence completes within a fixed time (two five-second timeouts), preventing hangs.

3. **Clean client experience**: Dropping the listener immediately ensures that clients racing the shutdown receive an immediate refusal rather than a confusing connection reset after a long timeout.

4. **Failure visibility**: Permanent connection failures surface as errors, allowing service supervisors to distinguish them from clean stops.

These properties make the daemon suitable for long-running service deployment under systemd or similar supervisors, while maintaining predictable behavior for IPC clients.