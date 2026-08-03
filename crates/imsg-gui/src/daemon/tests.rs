//! Real-socket tests for `status`/`stop` (same fake-broker approach as `imsg-broker-client`'s
//! own `query`/`tests.rs` — a real abstract-namespace listener, not a mock). `install`/
//! `uninstall` are intentionally untested against a real OS service manager here, matching
//! `imsg-service`'s own test suite (`crates/imsg-service/src/tests.rs`), which never calls
//! `install`/`uninstall`/`status` for real either — doing so would register an actual
//! systemd/launchd/etc. unit on the machine running the tests.

use crate::test_support::{bind_for, serve_one};

use super::*;

#[test]
fn level_maps_system_flag() {
    assert_eq!(level(true), service::ServiceLevel::System);
    assert_eq!(level(false), service::ServiceLevel::User);
}

#[test]
fn service_install_state_mirrors_service_state() {
    assert_eq!(
        ServiceInstallState::from(service::ServiceState::NotInstalled),
        ServiceInstallState::NotInstalled
    );
    assert_eq!(
        ServiceInstallState::from(service::ServiceState::Running),
        ServiceInstallState::Running
    );
    assert_eq!(
        ServiceInstallState::from(service::ServiceState::Stopped(Some("exited".to_owned()))),
        ServiceInstallState::Stopped(Some("exited".to_owned())),
    );
}

#[test]
fn uninstall_result_mirrors_uninstall_outcome() {
    assert_eq!(
        UninstallResult::from(service::UninstallOutcome::Uninstalled),
        UninstallResult::Uninstalled
    );
    assert_eq!(
        UninstallResult::from(service::UninstallOutcome::NotInstalled),
        UninstallResult::NotInstalled
    );
}

#[tokio::test]
async fn status_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:01";
    let resp = ipc::BrokerResponse::StatusInfo {
        state: ipc::SessionState::Active,
        device: addr.into(),
        persistent: true,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    assert_eq!(status(addr).await, Some(ipc::SessionState::Active));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn status_is_none_when_unreachable() {
    assert_eq!(status("TE:ST:00:00:02:02").await, None);
}

#[tokio::test]
async fn stop_reports_not_running_when_unreachable() -> anyhow::Result<()> {
    assert_eq!(stop("TE:ST:00:00:02:03").await?, StopOutcome::NotRunning);
    Ok(())
}

#[tokio::test]
async fn stop_reports_stopping_on_ok_response() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:04";
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, ipc::BrokerResponse::Ok));

    assert_eq!(stop(addr).await?, StopOutcome::Stopping);
    server.await??;
    Ok(())
}

#[tokio::test]
async fn stop_maps_error_response_to_rejected() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:02:05";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Error("no coordinator".to_owned())));

    let Err(err) = stop(addr).await else {
        return Err(anyhow::anyhow!("expected an error for an Error response"));
    };
    assert!(matches!(err, StopError::Rejected(msg) if msg == "no coordinator"));
    server.await??;
    Ok(())
}
