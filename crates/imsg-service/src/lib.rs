//! OS service-manager integration for imsg: register the persistent daemon
//! (`imsg daemon start --foreground`) as a systemd/launchd/OpenRC/rc.d/sc.exe
//! service and control it.
//!
//! Wraps the `service-manager` crate so callers (`cli`, the GUI) never need it as a
//! direct dependency — every type in this crate's public API is local to it.

// `pub mod` (not `mod`) is required: items inside are `pub(crate)`, and `pub(crate)` in a
// private module trips `redundant_pub_crate` while `pub` trips `unreachable_pub`.
pub mod identity;

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use service_manager::{
    RestartPolicy, ServiceInstallCtx, ServiceLabel, ServiceLevel as SmServiceLevel,
    ServiceManager as _, ServiceStartCtx, ServiceStatus as SmServiceStatus, ServiceStatusCtx,
    ServiceStopCtx, ServiceUninstallCtx, TypedServiceManager,
};
use thiserror::Error;

const LABEL_ORGANIZATION: &str = "imsg";
const LABEL_APPLICATION: &str = "daemon";

/// Whether the daemon service is registered system-wide or for the current user only.
///
/// System-level services on Linux/macOS typically require elevated privileges to
/// install; user-level services do not, but only run while the user session exists
/// (or lingers, on Linux with `loginctl enable-linger`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLevel {
    /// Registered for all users; typically requires elevated privileges to install.
    System,
    /// Registered for the current user only.
    User,
}

impl From<ServiceLevel> for SmServiceLevel {
    fn from(level: ServiceLevel) -> Self {
        match level {
            ServiceLevel::System => Self::System,
            ServiceLevel::User => Self::User,
        }
    }
}

/// Observed state of the registered daemon service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    /// No service is registered under imsg's label at the queried [`ServiceLevel`].
    NotInstalled,
    /// The service is registered and currently running.
    Running,
    /// The service is registered but not running, with a reason if the platform reports one.
    Stopped(Option<String>),
}

impl From<SmServiceStatus> for ServiceState {
    fn from(status: SmServiceStatus) -> Self {
        match status {
            SmServiceStatus::NotInstalled => Self::NotInstalled,
            SmServiceStatus::Running => Self::Running,
            SmServiceStatus::Stopped(reason) => Self::Stopped(reason),
        }
    }
}

/// Failure registering, controlling, or querying the daemon's OS service entry.
#[derive(Debug, Error)]
pub enum Error {
    /// The current executable path could not be resolved to populate the service's
    /// `ExecStart` (or platform equivalent).
    #[error("resolving current executable path: {0}")]
    CurrentExe(#[source] io::Error),
    /// No native service manager could be selected for this platform, or the
    /// requested [`ServiceLevel`] could not be applied to it.
    #[error("selecting service manager: {0}")]
    Manager(#[source] io::Error),
    /// The service manager rejected an install/uninstall/start/stop/status request.
    #[error("service operation failed: {0}")]
    Operation(#[source] io::Error),
    /// A `--system` install has no reliable "real user" to run as (e.g. a genuine root
    /// login rather than `sudo`) — refused rather than silently installing a root-run
    /// service that can't reach the installing user's config or keyring.
    #[error("refusing to install a system service as root — run via `sudo` as your normal user")]
    NoInvokingUser,
    /// Looking up the resolved invoking user's passwd entry failed.
    #[error("looking up invoking user: {0}")]
    UserLookup(#[source] nix::errno::Errno),
    /// The resolved invoking username has no passwd entry (deleted user, LDAP/SSSD hiccup).
    #[error("invoking user {0:?} has no passwd entry")]
    UnknownUser(String),
}

fn label() -> ServiceLabel {
    ServiceLabel {
        qualifier: None,
        organization: Some(LABEL_ORGANIZATION.to_owned()),
        application: LABEL_APPLICATION.to_owned(),
    }
}

fn manager(level: ServiceLevel) -> Result<TypedServiceManager, Error> {
    let mut manager = TypedServiceManager::native().map_err(Error::Manager)?;
    manager.set_level(level.into()).map_err(Error::Manager)?;
    Ok(manager)
}

/// Builds the `imsg daemon start --foreground [--device <addr>] [--config <path>]`
/// argument list passed to the installed service's `ExecStart`.
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

/// Resolves the real invoking user's home directory for a `--system` install.
///
/// For a caller resolving other config (e.g. a default `--device`) so it can read *their*
/// config instead of root's — the same resolution [`install`] itself uses for the service
/// unit's `User=`/`HOME=`.
///
/// # Errors
///
/// Returns [`Error::NoInvokingUser`] outside a `sudo` invocation (or a bare root login), or
/// [`Error::UserLookup`]/[`Error::UnknownUser`] if the resolved user's passwd entry can't be read.
pub fn invoking_home() -> Result<PathBuf, Error> {
    identity::invoking_user().map(|u| u.dir)
}

/// Registers the daemon with the native OS service manager, so it starts on boot
/// (or user login, for [`ServiceLevel::User`]) and restarts on failure.
///
/// `device`/`config_path` are forwarded as `--device`/`--config` to the service's
/// `imsg daemon start --foreground` invocation, matching the flags `imsg daemon start`
/// itself accepts. A [`ServiceLevel::System`] install additionally resolves the real
/// `sudo`-invoking user (see [`identity`]) and runs the service as them.
///
/// # Errors
///
/// Returns an error if the current executable path can't be resolved, a `--system` install
/// can't resolve a real invoking user, no native service manager is available, or the manager
/// rejects the install.
pub fn install(
    device: Option<&str>,
    config_path: Option<&Path>,
    level: ServiceLevel,
) -> Result<(), Error> {
    let program = env::current_exe().map_err(Error::CurrentExe)?;
    let (username, working_directory, environment) = identity::system_identity(level)?;
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
}

/// Unregisters the daemon service. A no-op (per the underlying service manager's
/// behavior) if it was never installed.
///
/// Stops the service first, best-effort: `uninstall` alone only removes the service
/// definition, leaving an already-running instance behind as an orphaned,
/// unsupervised process. A failed stop (e.g. nothing was running) doesn't block the
/// uninstall itself.
///
/// # Errors
///
/// Returns an error if no native service manager is available or it rejects the
/// uninstall.
pub fn uninstall(level: ServiceLevel) -> Result<(), Error> {
    let mgr = manager(level)?;
    let _ = mgr.stop(ServiceStopCtx { label: label() });
    mgr.uninstall(ServiceUninstallCtx { label: label() }).map_err(Error::Operation)
}

/// Starts the installed daemon service.
///
/// Reserved for a future service-management caller (e.g. a GUI panel) — no CLI command wires
/// this up yet, and it has only been exercised by the type-conversion unit tests in `tests.rs`,
/// not against a real `systemd`/`launchd`/`OpenRC`/`rc.d`/`sc.exe` unit. Verify against a real
/// service manager before adding the first caller.
///
/// # Errors
///
/// Returns an error if no native service manager is available, the service isn't
/// installed, or the manager rejects the start request.
pub fn start(level: ServiceLevel) -> Result<(), Error> {
    manager(level)?.start(ServiceStartCtx { label: label() }).map_err(Error::Operation)
}

/// Stops the installed daemon service.
///
/// Reserved for a future service-management caller (e.g. a GUI panel) — no CLI command wires
/// this up yet, and it has only been exercised by the type-conversion unit tests in `tests.rs`,
/// not against a real `systemd`/`launchd`/`OpenRC`/`rc.d`/`sc.exe` unit. Verify against a real
/// service manager before adding the first caller.
///
/// # Errors
///
/// Returns an error if no native service manager is available, the service isn't
/// installed, or the manager rejects the stop request.
pub fn stop(level: ServiceLevel) -> Result<(), Error> {
    manager(level)?.stop(ServiceStopCtx { label: label() }).map_err(Error::Operation)
}

/// Returns the installed daemon service's current state.
///
/// Reserved for a future service-management caller (e.g. a GUI panel) — no CLI command wires
/// this up yet, and it has only been exercised by the type-conversion unit tests in `tests.rs`,
/// not against a real `systemd`/`launchd`/`OpenRC`/`rc.d`/`sc.exe` unit. Verify against a real
/// service manager before adding the first caller.
///
/// # Errors
///
/// Returns an error if no native service manager is available or it fails to
/// report status (distinct from [`ServiceState::NotInstalled`], which is a normal
/// result, not an error).
pub fn status(level: ServiceLevel) -> Result<ServiceState, Error> {
    let status =
        manager(level)?.status(ServiceStatusCtx { label: label() }).map_err(Error::Operation)?;
    Ok(status.into())
}

#[cfg(test)]
mod tests;
