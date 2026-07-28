//! Round trip against the invoking user's *real* service manager, satisfying the
//! "verify against a real service manager before adding the first caller" note on
//! `status`.
//!
//! Verifies systemd only — the backend this machine runs. `status`'s absence detection is
//! per-backend and not uniformly reliable (`OpenRC` and `winsw` string-match their output),
//! so passing here does not establish the contract on other managers.
//!
//! Ignored by default: this registers and unregisters a unit under the invoking user's
//! session. Run deliberately with `--ignored`, never in CI.

use imsg_service::{
    install, status, uninstall, Error, ServiceLevel, ServiceState, UninstallOutcome,
};

const LEVEL: ServiceLevel = ServiceLevel::User;

#[test]
#[ignore = "mutates the invoking user's real service registration"]
fn install_status_uninstall_round_trip() -> Result<(), Error> {
    assert_eq!(
        status(LEVEL)?,
        ServiceState::NotInstalled,
        "precondition: no imsg unit may be registered before this runs"
    );

    install(Some("00:11:22:33:44:55"), None, LEVEL)?;
    let installed = status(LEVEL)?;
    assert!(
        matches!(installed, ServiceState::Stopped(_) | ServiceState::Running),
        "installed unit reported as {installed:?}"
    );

    assert_eq!(uninstall(LEVEL)?, UninstallOutcome::Uninstalled);
    assert_eq!(status(LEVEL)?, ServiceState::NotInstalled);

    // Finding 19: a repeat uninstall must report that nothing was there rather than
    // erroring, and must not claim to have removed anything.
    assert_eq!(uninstall(LEVEL)?, UninstallOutcome::NotInstalled);
    Ok(())
}
