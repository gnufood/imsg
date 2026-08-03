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
//!
//! Split into `lifecycle` (config/daemon/sync stage transitions) and `contacts` (the best-effort
//! contacts step) to stay under the module size ceiling.

mod contacts;
mod lifecycle;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use ipc::{BrokerResponse, SessionState, MAX_FRAME_LEN};
use secrecy::SecretBox;
use store::Store;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::test_support::bind_for;

use super::*;

const WAIT: Duration = Duration::from_secs(15);

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

fn snapshot(slot: &Mutex<Option<Store>>) -> Option<()> {
    slot.lock().unwrap_or_else(PoisonError::into_inner).as_ref().map(|_| ())
}

#[test]
fn state_defaults_to_initializing() {
    assert_eq!(GateState::default().status(), GateStatus::Initializing);
}
