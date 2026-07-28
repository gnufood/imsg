//! Public vocabulary types, split out of the crate root to keep it under the module ceiling.
//!
//! Each mirrors a `service-manager` type so the dependency stays out of callers' public APIs.

use service_manager::{ServiceLevel as SmServiceLevel, ServiceStatus as SmServiceStatus};

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

/// What an [`uninstall`](crate::uninstall) call actually did.
///
/// Distinguishes a removal from a no-op so callers don't report work that never happened —
/// uninstalling something absent is success, but it is not a removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallOutcome {
    /// Nothing was registered at this level; no change was made.
    NotInstalled,
    /// The registration was removed.
    Uninstalled,
}
