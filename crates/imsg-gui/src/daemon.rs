//! Daemon control: OS service registration (`imsg-service`, no broker) plus status/stop.
//!
//! Status/stop reuse `imsg-broker-client`'s already-generic status query — no domain-specific
//! request shape to extract, unlike `send`/`delete`/etc. Self-provisioning (checking for and, if
//! needed, launching a daemon) is a separate concern with its own module, [`provision`].

use std::path::Path;

use serde::{Deserialize, Serialize};

pub mod provision;

/// Maps the GUI's `system` flag to the underlying [`service::ServiceLevel`].
const fn level(system: bool) -> service::ServiceLevel {
    if system {
        service::ServiceLevel::System
    } else {
        service::ServiceLevel::User
    }
}

/// Registers the daemon with the native OS service manager. See [`service::install`] for what
/// `ExecStart`/environment it configures.
///
/// # Errors
///
/// Returns [`service::Error`] if no native service manager is available or it rejects the
/// install.
pub fn install(addr: &str, config_path: Option<&Path>, system: bool) -> Result<(), service::Error> {
    service::install(Some(addr), config_path, level(system))
}

/// Unregisters the daemon service. A no-op if it was never installed.
///
/// # Errors
///
/// Returns [`service::Error`] if no native service manager is available or it rejects the
/// uninstall.
pub fn uninstall(system: bool) -> Result<(), service::Error> {
    service::uninstall(level(system))
}

/// Returns the daemon's current session state, or `None` if nothing answers at `addr`.
pub async fn status(addr: &str) -> Option<ipc::SessionState> {
    broker_client::query_state(addr).await
}

/// Registration/run state of the daemon service at a given [`service::ServiceLevel`], mirrored
/// from [`service::ServiceState`] (that type has no `Serialize`/`specta::Type` derive of its own).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum ServiceInstallState {
    /// No service is registered under imsg's label at this level.
    NotInstalled,
    /// Registered and currently running.
    Running,
    /// Registered but not running, with a reason if the platform reports one.
    Stopped(Option<String>),
}

impl From<service::ServiceState> for ServiceInstallState {
    fn from(state: service::ServiceState) -> Self {
        match state {
            service::ServiceState::NotInstalled => Self::NotInstalled,
            service::ServiceState::Running => Self::Running,
            service::ServiceState::Stopped(reason) => Self::Stopped(reason),
        }
    }
}

/// Returns whether the daemon service is registered at the given level, and its run state if so.
///
/// # Errors
///
/// Returns [`service::Error`] if no native service manager is available or it fails to report
/// status.
pub fn service_status(system: bool) -> Result<ServiceInstallState, service::Error> {
    service::status(level(system)).map(ServiceInstallState::from)
}

/// Result of a [`stop`] request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum StopOutcome {
    /// Nothing was reachable at `addr` — a successful no-op, not an error (stopping something
    /// that isn't running is idempotent by design).
    NotRunning,
    /// The broker accepted the shutdown request.
    Stopping,
}

/// The broker itself rejected a [`stop`] request, or answered with a shape `Shutdown` should
/// never produce.
#[derive(Debug, thiserror::Error)]
pub enum StopError {
    /// The broker has no shutdown coordinator for this session (the ephemeral one-shot broker;
    /// only persistent/daemon mode supports `Shutdown`).
    #[error("{0}")]
    Rejected(String),
    /// A response shape `Shutdown` should never produce.
    #[error("unexpected broker response: {0:?}")]
    Unexpected(Box<ipc::BrokerResponse>),
}

/// Sends a graceful `Shutdown` request.
///
/// Any connection failure (including a plain "nothing listening") is treated as
/// [`StopOutcome::NotRunning`], not an error — mirrors the CLI's `daemon stop`. Unlike the
/// CLI's `run_stop`, this does not block waiting for the socket to go unreachable; the GUI's own
/// poll loop (see `internal/GUI.md`'s "Live updates") is expected to pick up the transition.
///
/// # Errors
///
/// Returns [`StopError`] if the broker answers but rejects the request or returns an
/// unexpected response shape.
pub async fn stop(addr: &str) -> Result<StopOutcome, StopError> {
    let Ok(resp) = broker_client::send_request(addr, ipc::BrokerRequest::Shutdown).await else {
        return Ok(StopOutcome::NotRunning);
    };
    match resp {
        ipc::BrokerResponse::Ok => Ok(StopOutcome::Stopping),
        ipc::BrokerResponse::Error(e) => Err(StopError::Rejected(e)),
        other => Err(StopError::Unexpected(Box::new(other))),
    }
}

#[cfg(test)]
mod tests;
