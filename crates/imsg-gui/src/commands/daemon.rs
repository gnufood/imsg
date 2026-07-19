//! `#[tauri::command]` shims over `crate::daemon`.
//!
//! No broker-request-shaped logic here, unlike `send`/`delete`/etc. — `install`/`uninstall` are
//! pure `imsg-service` calls, `status`/`stop` reuse `imsg-broker-client`'s already-generic
//! status query.

use std::path::PathBuf;

use ipc::SessionState;

use super::CommandError;

/// Registers the daemon with the native OS service manager.
///
/// # Errors
///
/// Returns [`CommandError`] if no native service manager is available or it rejects the
/// install.
// `String`/`PathBuf`, not `&str`/`&Path`: `#[tauri::command]` arguments are deserialized from
// the frontend's IPC call and must be owned.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub fn daemon_install(
    addr: String,
    config_path: Option<PathBuf>,
    system: bool,
) -> Result<(), CommandError> {
    Ok(crate::daemon::install(&addr, config_path.as_deref(), system)?)
}

/// Unregisters the daemon service. A no-op if it was never installed.
///
/// # Errors
///
/// Returns [`CommandError`] if no native service manager is available or it rejects the
/// uninstall.
#[tauri::command]
#[specta::specta]
pub fn daemon_uninstall(system: bool) -> Result<(), CommandError> {
    Ok(crate::daemon::uninstall(system)?)
}

/// Returns the daemon's current session state, or `None` if nothing answers at `addr`.
#[tauri::command]
#[specta::specta]
pub async fn daemon_status(addr: String) -> Option<SessionState> {
    crate::daemon::status(&addr).await
}

/// Returns the daemon service's registration/run state at the given level.
///
/// Untested against a real OS service manager for the same reason [`daemon_install`]/
/// [`daemon_uninstall`] are — see `crate::daemon`'s test doc.
///
/// # Errors
///
/// Returns [`CommandError`] if no native service manager is available or it fails to report
/// status.
#[tauri::command]
#[specta::specta]
pub fn daemon_service_status(
    system: bool,
) -> Result<crate::daemon::ServiceInstallState, CommandError> {
    Ok(crate::daemon::service_status(system)?)
}

/// Same underlying query as [`daemon_status`] — the CLI keeps `broker status`/`daemon status`
/// as separate subcommands (ephemeral vs. persistent broker), but the query itself has no
/// domain-specific shape to distinguish them.
#[tauri::command]
#[specta::specta]
pub async fn broker_status(addr: String) -> Option<SessionState> {
    crate::daemon::status(&addr).await
}

/// Sends a graceful `Shutdown` request.
///
/// # Errors
///
/// Returns [`CommandError`] if the broker answers but rejects the request or returns an
/// unexpected response shape.
#[tauri::command]
#[specta::specta]
pub async fn daemon_stop(addr: String) -> Result<crate::daemon::StopOutcome, CommandError> {
    Ok(crate::daemon::stop(&addr).await?)
}

/// Ensures a daemon is reachable at `addr`, self-provisioning one if none answers — the GUI's
/// only way to bring the daemon back after [`daemon_stop`], since `main.rs`'s own
/// self-provisioning (`ensure_running`) only ever runs once, at startup.
///
/// # Errors
///
/// Returns [`CommandError`] if `addr` is held by an ephemeral broker instead of a daemon, the
/// config can't be loaded, or spawning fails / the socket never becomes reachable.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub async fn daemon_restart(
    addr: String,
    config_path: Option<PathBuf>,
) -> Result<(), CommandError> {
    let cfg = config::load(config_path.clone())?;
    Ok(crate::daemon::provision::ensure_running(&cfg, Some(&addr), config_path).await?)
}

#[cfg(test)]
mod tests;
