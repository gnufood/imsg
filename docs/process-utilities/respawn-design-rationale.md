# Process Respawn Design Rationale

## Why a Shared Respawn Primitive?

The imsg workspace has three distinct call sites that all need to do the same fundamental thing: spawn the current executable as a background child process with specific arguments and redirected output. Before the `imsg-proc` crate existed, each call site carried its own near-identical implementation, differing only in the verb placed in `argv`, the stdout handling, and whether the child should be detached into its own process group.

These three call sites are:

1. **CLI daemon start (backgrounded)**: When a user runs `imsg daemon start` without `--foreground`, the CLI re-execs itself as `imsg daemon start --foreground` in a detached child. The parent waits for the child's socket to become reachable, then returns. This is the primary way users start a persistent daemon.

2. **CLI ephemeral broker**: When any CLI command needs to talk to a broker but none is running, it spawns an ephemeral one-shot broker via `imsg __broker_serve`. This child is not detached—it runs in the same process group as the parent and exits when the parent disconnects. The parent waits for socket readiness, then proceeds with its actual command.

3. **GUI self-provisioning**: The GUI application cannot run without a reachable daemon, but it also cannot shell out to a separate `imsg` binary (that would defeat the purpose of extracting `imsg-broker` as a CLI-agnostic library). Instead, when no daemon is reachable, the GUI re-execs itself with `--__daemon_foreground` to run the broker in-process. This is the only case where the launcher and the payload share the same binary.

The duplication wasn't just aesthetic. Each implementation had subtle differences in how it handled stdout redirection, process group creation, log file permissions, and argument assembly. Over time, these independent implementations became a maintenance burden: a bug fix in one had to be manually ported to the others, and the subtle behavioral differences made it difficult to reason about the system's overall process lifecycle.

## Design Tradeoffs of Self-Respawn

### Self-Respawn vs. Separate Binary

The most fundamental design choice is whether to spawn a separate binary or re-exec the current one. The CLI uses the same binary because it's already installed and available on `PATH—the same `imsg` binary that handles `send`, `sync`, and `daemon` also handles `__broker_serve` through argument dispatch.

The GUI, however, has no such guarantee. A Tauri application bundles its own executable, and there's no separate `imsg-gui` binary to invoke. More importantly, extracting `imsg-broker` as a library was specifically intended to make it usable by both the CLI and GUI without code duplication. Having the GUI shell out to a separate CLI binary would have defeated that architectural goal.

Self-respawn also simplifies deployment: there's only one binary to install, and it works regardless of how it was invoked. The tradeoff is that the child process must be able to recognize its role from argv—hence the use of distinct verbs like `--foreground`, `__broker_serve`, and `--__daemon_foreground` to distinguish the three modes.

### Process Group Detachment

On Unix systems, the `detach` parameter controls whether the child is placed in its own process group via `process_group(0)`. When detached, the child becomes a session leader and will survive the parent process exiting. This is essential for the daemon use case: the user runs `imsg daemon start` in a shell, the shell returns immediately, and the daemon continues running in the background.

For the ephemeral broker, detachment would be counterproductive. The broker should exit when the parent disconnects—if the parent crashes without closing the socket cleanly, the orphaned broker should also terminate rather than becoming a zombie. By keeping the ephemeral broker in the same process group, the parent's death automatically terminates the broker through the normal Unix process lifecycle.

The GUI's self-provisioned daemon is always detached, because the GUI window may be closed while the daemon continues running in the background. The user expects the daemon to persist even after closing the GUI.

### Stdio Redirection and Log File Handling

Every respawned child redirects its stdio to a log file, but the specifics differ:

- **stderr** is always captured to the log file, regardless of the `capture_stdout` flag. This ensures that any error output from the child is preserved for debugging, whether the child is a daemon or an ephemeral broker.

- **stdout** is only captured when `capture_stdout` is true. For the daemon, stdout capture is enabled because the log file is the primary debugging artifact. For the ephemeral broker, stdout is discarded because the broker produces no meaningful stdout output—it communicates entirely over the IPC socket—and discarding it avoids cluttering the log with irrelevant data.

The log file itself is created with `0o600` permissions on Unix. This is a deliberate security choice: the log may contain message content, device addresses, or other sensitive data that should not be readable by other users on the system. The parent directory is created if it doesn't exist, ensuring the log path is always valid.

### Argument Assembly and UTF-8 Validation

The `respawn_args` function assembles the child's argv by taking a verb (the command to run), a device address (always passed as `--device <addr>`), and an optional config path. The config path is the only place where UTF-8 validation is necessary: Unix argv is natively byte sequences, but the Rust `Command` API and the project's argument parsing both assume UTF-8 strings.

By validating the config path in `respawn_args` rather than letting it propagate to the child, the parent can fail fast with a clear error message rather than having the child crash with a confusing encoding error. This is especially important because the config path often comes from user input or environment variables that may have been set incorrectly.

### The Self-Respawn Contract

The respawn primitive establishes a clear contract between parent and child:

- The parent resolves `current_exe()` to find the binary to spawn—this ensures the child runs the same version as the parent, even if the original binary has been replaced on disk.
- The parent creates the log file before spawning, ensuring the child has a valid destination for its output from the very first line.
- The parent returns a `Child` handle to the caller, who is responsible for waiting on it or detaching it. This allows the caller to implement readiness probing (waiting for the socket to become reachable) before considering the spawn successful.
- The child receives its instructions entirely through argv—no shared memory, environment variables, or IPC channels are needed for the basic respawn case. This makes the mechanism simple, testable, and robust across process replacement boundaries.

## Alternatives Considered

Several alternatives were considered during the design of this module:

**Separate broker binary**: Instead of self-respawn, the GUI could have shipped a separate `imsg-broker` binary. This was rejected because it would have required the GUI to locate and invoke the CLI binary, creating a hard dependency between the two crates and complicating installation and packaging.

**Environment variable passing**: Instead of encoding everything in argv, the parent could set environment variables for the child to read. This was rejected because environment variables persist in the process environment and can leak to unrelated child processes. argv is explicit and scoped to this specific spawn.

**Library-based broker embedding**: Rather than re-executing, the GUI could link against `imsg-broker` and run it in a separate thread. This was rejected because the GUI needs to cleanly separate its lifecycle from the daemon's—the GUI window can close while the daemon continues, and restarting the daemon doesn't require restarting the GUI. Separate processes provide clearer isolation than threads.

**Generic process spawn utility**: A fully generic spawn utility that could handle any argv and any stdio configuration was considered, but it would have obscured the specific contract that imsg needs: the `--device` flag is mandatory, the config path is optional but must be validated, and the log file must be created before spawning. A generic utility would have required callers to get these details right every time, increasing the chance of bugs.

## Implications for Testing

The self-respawn mechanism poses testing challenges because it involves spawning a real process. The test suite addresses this by using the test harness itself as the spawned binary: under `cargo test`, `current_exe()` resolves to the test binary, which accepts `--help` and other standard arguments. The test verifies that the log file is created, that it receives output, and that the spawned process actually runs—exercising the full pipeline without mocking.

The test also verifies that both `capture_stdout` branches execute without panicking, even though the test harness's output doesn't meaningfully differ between them. This follows the project's existing precedent of not directly verifying process-group or fd-capture side effects in unit tests.