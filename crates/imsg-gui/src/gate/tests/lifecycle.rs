//! Config/daemon/sync stage transitions — parking, failure reporting, and retry semantics.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use ipc::{BrokerResponse, SyncReportDto};
use serial_test::serial;

use super::*;

#[test]
#[serial]
fn run_parks_at_awaiting_device_config_then_completes_once_persisted() -> anyhow::Result<()> {
    figment::Jail::expect_with(|jail| {
        jailed_env(jail, None);
        let addr = "AA:BB:CC:DD:EE:11";
        block_on(async move {
            let dir = tempfile::tempdir()?;
            let state = Arc::new(GateState::default());
            let mut rx = state.subscribe();
            let calls = Arc::new(AtomicUsize::new(0));
            let ready = spawn_run(&state, counting_opener(dir.path(), calls, 0));

            wait_for(&mut rx, |s| *s == GateStatus::AwaitingDeviceConfig, "AwaitingDeviceConfig")
                .await?;
            assert!(
                snapshot(&ready).is_none(),
                "store must not be handed off before the gate completes"
            );

            config::set_device_and_channels(addr, 4, 19)?;
            let listener = bind_for(addr)?;
            let server = tokio::spawn(serve_sequence(
                listener,
                vec![
                    status_info(true),
                    BrokerResponse::Text("synced".to_owned()),
                    BrokerResponse::ContactsSynced { report: SyncReportDto::UpToDate },
                ],
            ));
            state.proceed();

            wait_for(&mut rx, |s| *s == GateStatus::Ready, "Ready").await?;
            server.await??;
            assert!(snapshot(&ready).is_some(), "on_ready must receive the opened store");
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
    Ok(())
}

#[test]
#[serial]
fn run_reports_daemon_failure_then_recovers_on_proceed() -> anyhow::Result<()> {
    figment::Jail::expect_with(|jail| {
        let addr = "AA:BB:CC:DD:EE:12";
        jailed_env(jail, Some(addr));
        block_on(async move {
            let dir = tempfile::tempdir()?;
            let state = Arc::new(GateState::default());
            let mut rx = state.subscribe();
            let conflict = tokio::spawn(serve_sequence(bind_for(addr)?, vec![status_info(false)]));
            let ready = spawn_run(&state, counting_opener(dir.path(), Arc::default(), 0));

            let failed =
                wait_for(&mut rx, |s| matches!(s, GateStatus::Failed { .. }), "Failed").await?;
            conflict.await??;
            assert!(
                matches!(&failed, GateStatus::Failed { stage: GateStage::Daemon, .. }),
                "unexpected failure: {failed:?}"
            );

            let server = tokio::spawn(serve_sequence(
                bind_for(addr)?,
                vec![
                    status_info(true),
                    BrokerResponse::Text("synced".to_owned()),
                    BrokerResponse::ContactsSynced { report: SyncReportDto::UpToDate },
                ],
            ));
            state.proceed();

            wait_for(&mut rx, |s| *s == GateStatus::Ready, "Ready").await?;
            server.await??;
            assert!(snapshot(&ready).is_some());
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
    Ok(())
}

#[test]
#[serial]
fn run_reports_sync_failure_and_reuses_open_store_on_retry() -> anyhow::Result<()> {
    figment::Jail::expect_with(|jail| {
        let addr = "AA:BB:CC:DD:EE:13";
        jailed_env(jail, Some(addr));
        block_on(async move {
            let dir = tempfile::tempdir()?;
            let state = Arc::new(GateState::default());
            let mut rx = state.subscribe();
            let calls = Arc::new(AtomicUsize::new(0));
            let first = tokio::spawn(serve_sequence(
                bind_for(addr)?,
                vec![status_info(true), BrokerResponse::Error("map session down".to_owned())],
            ));
            let ready = spawn_run(&state, counting_opener(dir.path(), Arc::clone(&calls), 0));

            let failed =
                wait_for(&mut rx, |s| matches!(s, GateStatus::Failed { .. }), "Failed").await?;
            first.await??;
            assert!(
                matches!(&failed, GateStatus::Failed { stage: GateStage::Sync, .. }),
                "unexpected failure: {failed:?}"
            );

            let second = tokio::spawn(serve_sequence(
                bind_for(addr)?,
                vec![
                    status_info(true),
                    BrokerResponse::Text("synced".to_owned()),
                    BrokerResponse::ContactsSynced { report: SyncReportDto::UpToDate },
                ],
            ));
            state.proceed();

            wait_for(&mut rx, |s| *s == GateStatus::Ready, "Ready").await?;
            second.await??;
            assert_eq!(calls.load(Ordering::SeqCst), 1, "retry must reuse the already-open store");
            assert!(snapshot(&ready).is_some());
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
    Ok(())
}

#[test]
#[serial]
fn run_reports_store_open_failure_as_daemon_stage_and_reopens_on_retry() -> anyhow::Result<()> {
    figment::Jail::expect_with(|jail| {
        let addr = "AA:BB:CC:DD:EE:14";
        jailed_env(jail, Some(addr));
        block_on(async move {
            let dir = tempfile::tempdir()?;
            let state = Arc::new(GateState::default());
            let mut rx = state.subscribe();
            let calls = Arc::new(AtomicUsize::new(0));
            // One listener across both passes: probe, (open fails), probe, sync.
            let server = tokio::spawn(serve_sequence(
                bind_for(addr)?,
                vec![
                    status_info(true),
                    status_info(true),
                    BrokerResponse::Text("synced".to_owned()),
                    BrokerResponse::ContactsSynced { report: SyncReportDto::UpToDate },
                ],
            ));
            let ready = spawn_run(&state, counting_opener(dir.path(), Arc::clone(&calls), 1));

            let failed =
                wait_for(&mut rx, |s| matches!(s, GateStatus::Failed { .. }), "Failed").await?;
            assert!(
                matches!(&failed, GateStatus::Failed { stage: GateStage::Daemon, .. }),
                "unexpected failure: {failed:?}"
            );

            state.proceed();
            wait_for(&mut rx, |s| *s == GateStatus::Ready, "Ready").await?;
            server.await??;
            assert_eq!(calls.load(Ordering::SeqCst), 2, "failed open must be retried");
            assert!(snapshot(&ready).is_some());
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
    Ok(())
}
