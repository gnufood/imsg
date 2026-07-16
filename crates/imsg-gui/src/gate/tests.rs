//! Real-`Store`/real-socket tests for the startup gate loop (same fake-broker approach as
//! `daemon::provision`'s and `sync`'s own suites, no mocks).
//!
//! The store-opener is injected (see [`run`]'s doc) precisely so these tests can construct
//! `Store::open(path, fixed_test_key)` directly instead of going through `reads::open_store`'s
//! real OS keyring — keyring availability is host state (locked/absent Secret Service blocks on
//! a prompt), and every other `Store`-needing test in this crate already bypasses it the same
//! way to stay hermetic.
//!
//! Status assertions only ever target *parked* states (`AwaitingDeviceConfig`, `Failed`) or the
//! terminal `Ready` — `watch` coalesces intermediate values, so asserting a transient
//! `StartingDaemon`/`Syncing` would race the loop. Stage *order* is proven by the fake broker's
//! response sequencing (a sync answer can only be consumed after the daemon probe's) and by
//! `Failed`'s `stage` field.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerResponse, SessionState, MAX_FRAME_LEN};
use secrecy::SecretBox;
use serial_test::serial;
use store::Store;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::*;

const WAIT: Duration = Duration::from_secs(15);

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

/// Accepts and answers `responses.len()` connections in order — each broker call (daemon probe,
/// sync request) opens its own connection to the same abstract socket.
async fn serve_sequence(listener: Listener, responses: Vec<BrokerResponse>) -> anyhow::Result<()> {
    for resp in responses {
        let stream = listener.accept().await?;
        let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
        let mut framed = Framed::new(stream, codec);
        let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
        let _req: ipc::BrokerRequest = serde_json::from_slice(&frame)?;
        let bytes = Bytes::from(serde_json::to_vec(&resp)?);
        framed.send(bytes).await?;
    }
    Ok(())
}

fn status_info(persistent: bool) -> BrokerResponse {
    BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: "irrelevant".into(),
        persistent,
    }
}

fn jailed_env(jail: &mut figment::Jail, addr: Option<&str>) {
    let home = jail.directory().to_path_buf();
    if let Some(addr) = addr {
        jail.set_env("IMSG_DEVICE__ADDRESS", addr);
    }
    jail.set_env("HOME", home.to_str().unwrap_or_default());
}

/// Runs `fut` on a fresh runtime — `Jail::expect_with`'s closure isn't async.
fn block_on<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(fut)
}

/// Store-opener over a real temp-dir `SQLite` with a fixed test key (see module doc for why the
/// keyring is bypassed), counting calls and failing the first `fail_first` of them.
fn counting_opener(
    dir: &std::path::Path,
    calls: Arc<AtomicUsize>,
    fail_first: usize,
) -> impl for<'a> FnMut(&'a config::Config) -> OpenStoreFuture<'a> + Send + 'static {
    let path = dir.join("gate-test.db");
    move |_cfg| {
        let n = calls.fetch_add(1, Ordering::SeqCst);
        let path = path.clone();
        Box::pin(async move {
            if n < fail_first {
                return Err(crate::reads::Error::NoDataDir);
            }
            let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
            Ok(Store::open(path, key).await?)
        })
    }
}

/// Spawns [`run`] with the given opener, returning the slot `on_ready`'s `Store` lands in.
fn spawn_run<F>(state: &Arc<GateState>, opener: F) -> Arc<Mutex<Option<Store>>>
where
    F: for<'a> FnMut(&'a config::Config) -> OpenStoreFuture<'a> + Send + 'static,
{
    let ready: Arc<Mutex<Option<Store>>> = Arc::default();
    let sink = Arc::clone(&ready);
    let state = Arc::clone(state);
    tokio::spawn(async move {
        run(&state, None, opener, move |db| {
            *sink.lock().unwrap_or_else(PoisonError::into_inner) = Some(db);
        })
        .await;
    });
    ready
}

async fn wait_for(
    rx: &mut tokio::sync::watch::Receiver<GateStatus>,
    pred: impl FnMut(&GateStatus) -> bool,
    what: &str,
) -> anyhow::Result<GateStatus> {
    let status = tokio::time::timeout(WAIT, rx.wait_for(pred))
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for {what}"))??
        .clone();
    Ok(status)
}

#[test]
fn state_defaults_to_initializing() {
    assert_eq!(GateState::default().status(), GateStatus::Initializing);
}

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
                vec![status_info(true), BrokerResponse::Text("synced".to_owned())],
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
                vec![status_info(true), BrokerResponse::Text("synced".to_owned())],
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
                vec![status_info(true), BrokerResponse::Text("synced".to_owned())],
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

fn snapshot(slot: &Mutex<Option<Store>>) -> Option<()> {
    slot.lock().unwrap_or_else(PoisonError::into_inner).as_ref().map(|_| ())
}
