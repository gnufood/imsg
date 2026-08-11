# Installing the Daemon as a System Service

This walkthrough traces the flow of registering the imsg daemon as a persistent OS service that starts on boot (or user login) and restarts automatically on failure. The flow begins when a user invokes the `imsg daemon install` command and completes with the service registered in the native service manager.

## Operation Overview

**Operation**: Register the imsg daemon with the OS service manager as a persistent, auto-starting service.

**Starting conditions**:
- The user has executed `imsg daemon install` or `imsg daemon install --system`
- The CLI has parsed the command and extracted any `--device` or `--config` arguments

**Terminal state**: The daemon service is registered with the platform's native service manager (systemd on Linux, launchd on macOS, OpenRC, rc.d, or sc.exe on Windows), configured to start on boot and restart on failure.

**Participating components**:
- CLI command handler (`daemon.rs`) — parses and dispatches the install command
- Service module (`imsg-service`) — orchestrates the install operation
- Identity module — resolves the real invoking user for `--system` installs
- Types module — provides public vocabulary types
- `service-manager` crate — platform abstraction for OS service managers

## Flow Phases

### Phase 1: CLI Command Dispatch

The flow begins in the CLI's daemon command handler. When the user invokes `imsg daemon install` with optional flags, the command is dispatched to the `dispatch` function in `crates/cli/src/commands/daemon.rs`.

```rust
DaemonCmd::Install { system } => {
    let addr = match device {
        Some(d) => d.to_owned(),
        None => default_device(config_path.clone(), system)?,
    };
    Ok(Some(install(&addr, config_path.as_deref(), system)?))
}
```

The CLI resolves the Bluetooth device address in one of two ways:
- If `--device` was passed explicitly, that value is used
- If not provided, `default_device()` is called to read from the user's config

For a `--system` install without an explicit `--config` path, the CLI uses `service::invoking_home()` to resolve the real invoking user's home directory, ensuring config is read from their `~/.config/imsg` rather than root's.

### Phase 2: Service Level Resolution

The CLI maps the `--system` flag to the appropriate `ServiceLevel`:

```rust
const fn level(system: bool) -> service::ServiceLevel {
    if system {
        service::ServiceLevel::System
    } else {
        service::ServiceLevel::User
    }
}
```

- `ServiceLevel::User` — the service runs for the current user only, starting when that user logs in (or via `loginctl enable-linger` on Linux for a headless user session)
- `ServiceLevel::System` — the service runs system-wide, typically requiring elevated privileges to install

### Phase 3: Identity Resolution for System Services

When `ServiceLevel::System` is requested, the service module must determine which user the service should run as. This is critical because a system service running as root cannot access the installing user's config, keyring, or Bluetooth pairing.

The identity resolution happens in `crates/imsg-service/src/identity.rs`. The `system_identity()` function is called:

```rust
pub(crate) fn system_identity(level: ServiceLevel) -> Result<ServiceIdentity, Error> {
    if level != ServiceLevel::System {
        return Ok((None, None, None));
    }
    let user = invoking_user()?;
    let config_home = user.dir.join(".config").to_string_lossy().into_owned();
    let env = vec![
        ("HOME".to_owned(), user.dir.to_string_lossy().into_owned()),
        ("XDG_CONFIG_HOME".to_owned(), config_home),
    ];
    Ok((Some(user.name), Some(user.dir), Some(env)))
}
```

The `invoking_user()` function resolves the actual user by examining environment variables set by `sudo`:

1. If `sudo` was used, `SUDO_USER` and `SUDO_UID` are set. The function validates that `SUDO_UID` matches the real UID of `SUDO_USER` from the passwd database — this prevents a malicious `SUDO_USER` environment variable from being trusted.
2. If the `sudo` pair is untrusted or absent, it falls back to `$USER`.
3. It refuses to return "root" — a genuine root login (not via sudo) would result in a service that cannot access any user's config, which defeats the purpose.

The returned identity includes:
- **username**: the account the service will run as
- **working_directory**: that user's home directory
- **environment**: `HOME` and `XDG_CONFIG_HOME` set to point to the user's directories

For `ServiceLevel::User`, all three values are `None` — the service runs as the user who started it, which is already the correct account.

### Phase 4: Executable and Command-Line Resolution

The service module resolves the current executable path to populate the service's `ExecStart` (or platform equivalent):

```rust
let program = env::current_exe().map_err(Error::CurrentExe)?;
```

It also constructs the arguments that the service manager will invoke:

```rust
fn start_foreground_args(device: Option<&str>, config_path: Option<&Path>) -> Vec<OsString> {
    let mut args =
        vec![OsString::from("daemon"), OsString::from("start"), OsString::from("--foreground")];
    if let Some(addr) = device {
        args.push(OsString::from("--device"));
        args.push(OsString::from(addr));
    }
    if let Some(path) = config_path {
        args.push(OsString::from("--config"));
        args.push(path.as_os_str().to_owned());
    }
    args
}
```

The `--foreground` flag is essential: it tells the daemon to run in the foreground rather than detaching, which is the expected behavior when a service manager spawns the process. The `--device` address is baked into the service definition because a `--system` service running as root has no user config to fall back on at startup time.

### Phase 5: Service Manager Selection and Installation

The service module selects the native service manager for the current platform:

```rust
fn manager(level: ServiceLevel) -> Result<TypedServiceManager, Error> {
    let mut manager = TypedServiceManager::native().map_err(Error::Manager)?;
    manager.set_level(level.into()).map_err(Error::Manager)?;
    Ok(manager)
}
```

`TypedServiceManager::native()` auto-detects the platform's service manager:
- **systemd** on Linux systems with systemd
- **launchd** on macOS
- **OpenRC** on Alpine and Gentoo
- **rc.d** on FreeBSD and OpenBSD
- **sc.exe** on Windows

The install context is built with all resolved values:

```rust
let ctx = ServiceInstallCtx {
    label: label(),
    program,
    args: start_foreground_args(device, config_path),
    contents: None,
    username,
    working_directory,
    environment,
    autostart: true,
    restart_policy: RestartPolicy::default(),
};
manager(level)?.install(ctx).map_err(Error::Operation)
```

Key fields:
- **label**: `imsg.daemon` — the service's unique identifier in the service manager
- **program**: path to the current `imsg` executable
- **args**: `daemon start --foreground [--device <addr>] [--config <path>]`
- **username**: resolved user for `--system` installs, `None` for `--user`
- **working_directory**: user's home directory for `--system`
- **environment**: `HOME` and `XDG_CONFIG_HOME` for `--system`
- **autostart**: `true` — the service starts on boot/login
- **restart_policy**: `default()` — typically "restart on failure with exponential backoff"

The service manager writes the appropriate unit/file/plist and enables the service. On systemd, this creates a unit file in `/etc/systemd/system/` (for `--system`) or `~/.config/systemd/user/` (for `--user`), then runs `systemctl daemon-reload` and `systemctl enable`.

## Failure Conditions

Several failure modes can interrupt this flow:

- **No invoking user for `--system`**: If the process is running as root without sudo, or the sudo environment variables are malformed, `Error::NoInvokingUser` is returned. This prevents silently installing a root-run service that cannot access any user's config.

- **User lookup failure**: If the resolved username has no passwd entry (deleted user, LDAP/SSSD issue), `Error::UserLookup` or `Error::UnknownUser` is returned.

- **No native service manager**: If the platform has no supported service manager, `Error::Manager` is returned.

- **Executable resolution failure**: If `env::current_exe()` fails, `Error::CurrentExe` is returned.

- **Service manager rejects install**: Permission denied (insufficient privileges for `--system`), label collision, or other manager-specific errors surface as `Error::Operation`.

## Resulting State

After a successful install:

1. The daemon service is registered with the OS service manager under the label `imsg.daemon`
2. The service is enabled to start on boot (for `--system`) or on user login (for `--user`)
3. The service is configured to restart automatically on failure
4. The service will run `imsg daemon start --foreground` with the resolved `--device` address and any explicit `--config` path
5. For `--system` installs, the service runs as the sudo-invoking user with their `HOME` and `XDG_CONFIG_HOME` environment, giving it access to their config and keyring

The service can be queried with `imsg daemon status` or the platform's native tool (`systemctl status imsg.daemon`, `launchctl list`, etc.), and controlled with `imsg daemon start`/`stop` or the equivalent native commands.