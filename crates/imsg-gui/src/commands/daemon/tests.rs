//! Real-socket tests for `daemon_status`/`broker_status`/`daemon_stop` (same fake-broker
//! approach as `crate::daemon`'s own tests). `daemon_install`/`daemon_uninstall` are
//! intentionally untested here for the same reason `crate::daemon::{install,uninstall}` are —
//! see that module's test doc. `daemon_restart` reuses `daemon/provision/tests.rs`'s
//! `figment::Jail` + fresh-runtime approach, since it (unlike the other commands here) loads a
//! real `Config`.

use serial_test::serial;

use crate::test_support::{bind_for, serve_one};

use super::*;

/// Loads a real minimal `Config` inside an env-isolated `figment::Jail` — mirrors
/// `daemon/provision/tests.rs`'s helper of the same shape.
fn jailed_env(jail: &mut figment::Jail) {
    let home = jail.directory().to_path_buf();
    jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
    jail.set_env("HOME", home.to_str().unwrap_or_default());
}

/// Runs `fut` on a fresh runtime — `Jail::expect_with`'s closure isn't async, so tests need
/// their own runtime to drive one. Callers convert the `anyhow::Error` into the closure's
/// required `figment::Error` themselves (large by figment's own design, not ours to box; same
/// reasoning as `daemon/provision/tests.rs`'s helper of the same shape).
fn run<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(fut)
}

#[tokio::test]
async fn daemon_status_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:03:01";
    let resp = ipc::BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: addr.into(),
        persistent: true,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    assert_eq!(daemon_status(addr.to_owned()).await, Some(SessionState::Active));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn broker_status_reports_session_state() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:03:02";
    let resp = ipc::BrokerResponse::StatusInfo {
        state: SessionState::Connecting,
        device: addr.into(),
        persistent: false,
    };
    let listener = bind_for(addr)?;
    let server = tokio::spawn(serve_one(listener, resp));

    assert_eq!(broker_status(addr.to_owned()).await, Some(SessionState::Connecting));
    server.await??;
    Ok(())
}

#[tokio::test]
async fn daemon_stop_reports_not_running_when_unreachable() -> anyhow::Result<()> {
    let outcome = daemon_stop("TE:ST:00:00:03:03".to_owned()).await?;
    assert_eq!(outcome, crate::daemon::StopOutcome::NotRunning);
    Ok(())
}

#[tokio::test]
async fn daemon_stop_maps_error_response_to_command_error() -> anyhow::Result<()> {
    let addr = "TE:ST:00:00:03:04";
    let listener = bind_for(addr)?;
    let server =
        tokio::spawn(serve_one(listener, ipc::BrokerResponse::Error("no coordinator".to_owned())));

    let result = daemon_stop(addr.to_owned()).await;
    assert!(result.is_err());
    server.await??;
    Ok(())
}

#[test]
#[serial]
fn daemon_restart_is_ok_when_daemon_already_running() {
    figment::Jail::expect_with(|jail| {
        jailed_env(jail);
        run(async move {
            let addr = "TE:ST:00:00:03:05";
            let resp = ipc::BrokerResponse::StatusInfo {
                state: SessionState::Active,
                device: addr.into(),
                persistent: true,
            };
            let listener = bind_for(addr)?;
            let server = tokio::spawn(serve_one(listener, resp));

            daemon_restart(addr.to_owned(), None).await?;

            server.await??;
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
}

#[test]
#[serial]
fn daemon_restart_maps_ephemeral_conflict_to_command_error() {
    figment::Jail::expect_with(|jail| {
        jailed_env(jail);
        run(async move {
            let addr = "TE:ST:00:00:03:06";
            let resp = ipc::BrokerResponse::StatusInfo {
                state: SessionState::Active,
                device: addr.into(),
                persistent: false,
            };
            let listener = bind_for(addr)?;
            let server = tokio::spawn(serve_one(listener, resp));

            let result = daemon_restart(addr.to_owned(), None).await;
            anyhow::ensure!(result.is_err(), "expected an ephemeral-conflict CommandError");

            server.await??;
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
}
