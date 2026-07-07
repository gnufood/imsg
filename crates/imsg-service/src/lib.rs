//! OS service-manager integration for imsg: register the persistent daemon
//! (`imsg daemon start --foreground`) as a systemd/launchd/OpenRC/rc.d/sc.exe
//! service and control it.
//!
//! Wraps the `service-manager` crate so callers (`cli`, the GUI) never need it as a
//! direct dependency — every type in this crate's public API is local to it.

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;

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

/// Registers the daemon with the native OS service manager, so it starts on boot
/// (or user login, for [`ServiceLevel::User`]) and restarts on failure.
///
/// `device`/`config_path` are forwarded as `--device`/`--config` to the service's
/// `imsg daemon start --foreground` invocation, matching the flags `imsg daemon start`
/// itself accepts.
///
/// # Errors
///
/// Returns an error if the current executable path can't be resolved, no native
/// service manager is available, or the manager rejects the install.
pub fn install(
    device: Option<&str>,
    config_path: Option<&Path>,
    level: ServiceLevel,
) -> Result<(), Error> {
    let program = env::current_exe().map_err(Error::CurrentExe)?;
    let ctx = ServiceInstallCtx {
        label: label(),
        program,
        args: start_foreground_args(device, config_path),
        contents: None,
        username: None,
        working_directory: None,
        environment: None,
        autostart: true,
        restart_policy: RestartPolicy::default(),
    };
    manager(level)?.install(ctx).map_err(Error::Operation)
}

/// Unregisters the daemon service. A no-op (per the underlying service manager's
/// behavior) if it was never installed.
///
/// # Errors
///
/// Returns an error if no native service manager is available or it rejects the
/// uninstall.
pub fn uninstall(level: ServiceLevel) -> Result<(), Error> {
    manager(level)?.uninstall(ServiceUninstallCtx { label: label() }).map_err(Error::Operation)
}

/// Starts the installed daemon service.
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
/// # Errors
///
/// Returns an error if no native service manager is available, the service isn't
/// installed, or the manager rejects the stop request.
pub fn stop(level: ServiceLevel) -> Result<(), Error> {
    manager(level)?.stop(ServiceStopCtx { label: label() }).map_err(Error::Operation)
}

/// Returns the installed daemon service's current state.
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
mod tests {
    use super::*;

    /// `ExecStart` args always lead with the foreground subcommand, regardless of
    /// which optional overrides are present.
    #[test]
    fn start_foreground_args_base_case() {
        assert_eq!(
            start_foreground_args(None, None),
            vec![OsString::from("daemon"), OsString::from("start"), OsString::from("--foreground"),]
        );
    }

    /// `--device`/`--config` are appended in order, matching `imsg daemon start`'s own
    /// flag names so the installed service invokes the identical CLI surface.
    #[test]
    fn start_foreground_args_with_overrides() {
        let args = start_foreground_args(Some("AA:BB:CC:DD:EE:FF"), Some(Path::new("/tmp/c.toml")));
        assert_eq!(
            args,
            vec![
                OsString::from("daemon"),
                OsString::from("start"),
                OsString::from("--foreground"),
                OsString::from("--device"),
                OsString::from("AA:BB:CC:DD:EE:FF"),
                OsString::from("--config"),
                OsString::from("/tmp/c.toml"),
            ]
        );
    }

    /// Round-trips through [`SmServiceStatus`] preserve the stopped-reason payload.
    #[test]
    fn service_state_from_status_preserves_stopped_reason() {
        assert_eq!(ServiceState::from(SmServiceStatus::NotInstalled), ServiceState::NotInstalled);
        assert_eq!(ServiceState::from(SmServiceStatus::Running), ServiceState::Running);
        assert_eq!(
            ServiceState::from(SmServiceStatus::Stopped(Some("crashed".to_owned()))),
            ServiceState::Stopped(Some("crashed".to_owned()))
        );
    }
}
