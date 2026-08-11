# Service Management API Reference

This module provides the public API for registering the imsg daemon as an OS service and controlling its lifecycle through the native service manager (systemd, launchd, OpenRC, rc.d, or sc.exe on Windows).

## Service Label

All service operations target a fixed label defined by the following components:

| Field | Value |
|-------|-------|
| Organization | `imsg` |
| Application | `daemon` |
| Qualifier | None (unset) |

The label is constructed internally by the `label()` function and cannot be configured by callers. This fixed identity ensures that all service management operations refer to the same registered service.

## Service Levels

The `ServiceLevel` enum specifies the scope of the service registration:

```rust
pub enum ServiceLevel {
    System,  // Registered for all users; requires elevated privileges
    User,    // Registered for the current user only
}
```

- **System**: The service starts at boot and runs with system-wide privileges. On Linux and macOS, installation typically requires elevated privileges. The service runs as a specific user (see Identity Resolution below), not as root.
- **User**: The service starts when the user logs in (or when the user session lingers, on Linux with `loginctl enable-linger`). No elevated privileges are required for installation.

## Operations

### install

```rust
pub fn install(
    device: Option<&str>,
    config_path: Option<&Path>,
    level: ServiceLevel,
) -> Result<(), Error>
```

Registers the daemon with the native OS service manager. The service is configured to start automatically on boot (or user login for `ServiceLevel::User`) and restart on failure.

**Parameters:**
- `device`: Optional Bluetooth device address (`--device` flag passed to the daemon)
- `config_path`: Optional path to a configuration file (`--config` flag passed to the daemon)
- `level`: The service scope (`System` or `User`)

**Execution flow:**
1. Resolves the current executable path to populate the service's `ExecStart`
2. For `ServiceLevel::System`, resolves the invoking user's identity (see Identity Resolution)
3. Constructs the start command as `imsg daemon start --foreground [--device <addr>] [--config <path>]`
4. Passes the constructed context to the underlying service manager

**Behavior by level:**
- `ServiceLevel::User`: The service runs as the logged-in user with no explicit user specification
- `ServiceLevel::System`: The service runs as the resolved invoking user (not root), with `HOME` and `XDG_CONFIG_HOME` environment variables set to that user's home directory

### uninstall

```rust
pub fn uninstall(level: ServiceLevel) -> Result<UninstallOutcome, Error>
```

Unregisters the daemon service from the native OS service manager.

**Atomicity:** This operation is not atomic. A service installed between the status check and removal is still removed; a service removed in that window surfaces as an error rather than a no-op.

**Stop behavior:** The operation first attempts to stop the service (best-effort). A failed stop (e.g., nothing was running) does not block the uninstall. The service definition is removed regardless of whether the service was running.

**Return values:**
- `UninstallOutcome::Uninstalled`: The service was registered and its definition was removed
- `UninstallOutcome::NotInstalled`: No service was registered at the queried level; no change was made

Uninstalling something absent returns `NotInstalled`, not an error. This matches the convention that undoing a state you're already in is success.

### start

```rust
pub fn start(level: ServiceLevel) -> Result<(), Error>
```

Starts a registered service. This function is currently unused; it is reserved for future service-management callers such as a GUI panel.

**Prerequisites:** The service must be installed, or the operation fails.

### stop

```rust
pub fn stop(level: ServiceLevel) -> Result<(), Error>
```

Stops a running service. This function is currently unused; it is reserved for future service-management callers such as a GUI panel.

**Prerequisites:** The service must be installed and running, or the operation fails.

### status

```rust
pub fn status(level: ServiceLevel) -> Result<ServiceState, Error>
```

Queries the current state of the registered service. This function is currently unused; it is reserved for future service-management callers such as a GUI panel.

**Return values:**
- `ServiceState::NotInstalled`: No service is registered under imsg's label at the queried level
- `ServiceState::Running`: The service is registered and currently running
- `ServiceState::Stopped(reason)`: The service is registered but not running; `reason` is present if the platform reports one

### invoking_home

```rust
pub fn invoking_home() -> Result<PathBuf, Error>
```

Resolves the real invoking user's home directory. This is used internally for `ServiceLevel::System` installs to determine the service's working directory and environment, but is also exposed for callers that need to resolve other config (e.g., a default `--device`) so they can read the invoking user's config instead of root's.

## Identity Resolution

For `ServiceLevel::System` installs, the service must run as the real invoking user (not root), because the user's config, keyring, and Bluetooth pairing data all live under their account.

The resolution process:

1. Checks for `SUDO_USER` environment variable (set when the caller invokes via `sudo`)
2. Corroborates `SUDO_USER` against `SUDO_UID` by looking up the claimed username's real UID and comparing it to the parsed `SUDO_UID`
3. If corroborated, uses the `SUDO_USER` value; otherwise falls back to `$USER`
4. Rejects a bare `root` result in all cases — a genuine root login (no `sudo`) would otherwise silently install a system service that runs as root, defeating the purpose of user resolution

The resolved user's passwd entry is then looked up to obtain:
- Username (for the service's `User=` directive)
- Home directory (for `HOME` and `XDG_CONFIG_HOME` environment variables, and as the working directory)

## Types

### ServiceState

```rust
pub enum ServiceState {
    NotInstalled,
    Running,
    Stopped(Option<String>),
}
```

Represents the observed state of the registered daemon service.

### UninstallOutcome

```rust
pub enum UninstallOutcome {
    NotInstalled,
    Uninstalled,
}
```

Distinguishes a removal from a no-op so callers don't report work that never happened.

### Error

```rust
pub enum Error {
    CurrentExe(#[source] io::Error),
    Manager(#[source] io::Error),
    Operation(#[source] io::Error),
    NoInvokingUser,
    UserLookup(#[source] nix::errno::Errno),
    UnknownUser(String),
}
```

| Variant | Condition |
|---------|-----------|
| `CurrentExe` | The current executable path could not be resolved to populate the service's `ExecStart` |
| `Manager` | No native service manager could be selected for the platform, or the requested `ServiceLevel` could not be applied |
| `Operation` | The service manager rejected an install/uninstall/start/stop/status request |
| `NoInvokingUser` | A `--system` install has no reliable "real user" to run as (e.g., a genuine root login rather than `sudo`) |
| `UserLookup` | Looking up the resolved invoking user's passwd entry failed |
| `UnknownUser` | The resolved invoking username has no passwd entry (deleted user, LDAP/SSSS hiccup) |

## Platform Support

The underlying `service-manager` crate provides abstraction over the following native service managers:

| Platform | Service Manager |
|----------|-----------------|
| Linux (most distributions) | systemd |
| macOS | launchd |
| OpenRC (e.g., Alpine, Gentoo) | OpenRC |
| FreeBSD, NetBSD | rc.d |
| Windows | sc.exe (Service Control Manager via winsw) |

The `TypedServiceManager::native()` call automatically selects the appropriate manager for the current platform.

## Dependencies

This crate wraps the `service-manager` crate so that callers (`cli`, the GUI) never need it as a direct dependency. Every type in this crate's public API is local to it, providing a stable surface that does not leak the underlying dependency's types into the public interface.