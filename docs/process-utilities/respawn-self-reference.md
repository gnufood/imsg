# respawn_self API Reference

Re-executes the current binary as a background child process with logging and optional detachment.

## Overview

The `respawn_self` function spawns a new instance of the currently running executable with modified command-line arguments. It is a shared primitive used across the workspace for "re-invoke myself as a background process" operations, including the CLI's backgrounded `daemon start`, the CLI's ephemeral one-shot broker, and the GUI's headless self-provisioned daemon.

## Function Signature

```rust
pub async fn respawn_self(
    verb: &[&str],
    addr: &str,
    config_path: Option<&Path>,
    log_path: &Path,
    capture_stdout: bool,
    detach: bool,
) -> Result<Child>
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `verb` | `&[&str]` | Command verb and any initial arguments to pass to the new process. |
| `addr` | `&str` | Device address string (e.g., MAC address) to pass via `--device`. |
| `config_path` | `Option<&Path>` | Optional path to a configuration file; if present, passed as `--config`. |
| `log_path` | `&Path` | Path to the log file where stdout and stderr are redirected. |
| `capture_stdout` | `bool` | If `true`, redirects stdout to the log file; if `false`, discards stdout. |
| `detach` | `bool` | If `true`, moves the child into its own process group (Unix) so it survives the parent exiting. |

## Return Value

Returns `Result<Child>`, where `Child` is a Tokio `process::Child` handle to the spawned subprocess. The caller may wait on or manage the child process as needed.

## Behavior

### Argument Assembly

The function builds the child process arguments as follows:

1. Copies the `verb` slice elements into a new `Vec<String>`.
2. Appends `--device` followed by the `addr` string.
3. If `config_path` is `Some(path)`:
   - Appends `--config` followed by the path converted to a UTF-8 string.

The resulting argument vector is: `[verb..., "--device", addr]` or `[verb..., "--device", addr, "--config", config_path]`.

### Log File Creation

The log file is created with the following behavior:

- If the parent directory does not exist, it is created recursively.
- The file is created with `create(true).write(true).truncate(true)`.
- On Unix, the file permissions are set to `0o600` (owner read/write only) because logs may contain sensitive message content.
- On non-Unix platforms, default permissions apply.

### Standard I/O Redirection

| Stream | `capture_stdout = false` | `capture_stdout = true` |
|--------|--------------------------|-------------------------|
| stdin | `Stdio::null()` | `Stdio::null()` |
| stdout | `Stdio::null()` | Redirected to log file |
| stderr | Redirected to log file | Redirected to log file |

Stderr is always redirected to the log file. Stdout is either discarded or captured depending on the `capture_stdout` parameter.

### Process Detachment (Unix)

On Unix systems, when `detach` is `true`, the child process is placed into a new process group by calling `process_group(0)`. This causes the child to become an orphan relative to the parent process — it will not receive SIGHUP when the parent exits and will continue running in the background.

This parameter has no effect on non-Unix platforms.

## Errors

The function returns an error in the following conditions:

- **`config_path` is not valid UTF-8**: If a config path is provided but cannot be converted to a UTF-8 string, the function returns an error containing "UTF-8".
- **Current executable path cannot be resolved**: If `std::env::current_exe()` fails.
- **Log directory creation fails**: If the parent directory of `log_path` cannot be created.
- **Log file cannot be opened**: If the log file cannot be created or opened for writing.
- **Log file handle duplication fails**: If `capture_stdout` is `true` and the log file handle cannot be duplicated for stdout redirection.
- **Subprocess spawn fails**: If the child process cannot be spawned.

## Examples

### Spawning a detached background daemon

```rust
let child = respawn_self(
    &["daemon", "start"],
    "AA:BB:CC:DD:EE:FF",
    Some(Path::new("/etc/imsg.toml")),
    Path::new("/var/log/imsg/daemon.log"),
    false,  // don't capture stdout
    true,   // detach into own process group
).await?;
```

### Spawning with stdout captured

```rust
let child = respawn_self(
    &["__broker_serve"],
    "AA:BB:CC:DD:EE:FF",
    None,
    Path::new("/tmp/broker.log"),
    true,   // capture stdout to log
    false,  // don't detach
).await?;
```

## See Also

- [`respawn_args`](#argument-assembly) — Pure function for building the argument vector.
- [`open_log`](#log-file-creation) — Helper for creating log files with appropriate permissions.