//! `classify` is pure and tested directly. `ensure_running`'s `AlreadyRunning`/
//! `EphemeralConflict` branches return before ever spawning, so they're real-socket tested the
//! same way as `daemon/tests.rs`'s `status`/`stop` tests — real `Config` via `figment::Jail`
//! (same isolation approach `config/tests.rs` uses, `#[serial]` since `Jail` mutates
//! process-global env vars). The `Unreachable` (spawn) branch isn't — the spawn itself is now
//! `imsg_proc::respawn_self`, tested there.

use bytes::Bytes;
use futures::{SinkExt as _, StreamExt as _};
use interprocess::local_socket::tokio::prelude::*;
use interprocess::local_socket::tokio::Listener;
use interprocess::local_socket::ListenerOptions;
use ipc::{BrokerResponse, SessionState, MAX_FRAME_LEN};
use serial_test::serial;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::*;

/// Loads a real minimal `Config` inside an env-isolated `figment::Jail` — `device.address` is
/// required with no default, but unused by the branches under test (`ensure_running` is always
/// called with an explicit `device`, short-circuiting the `cfg.device.address()` fallback).
// `figment::Error` is what `Jail::expect_with`'s closure return type is fixed to — not ours to box.
#[allow(clippy::result_large_err)]
fn load_test_config(jail: &mut figment::Jail) -> Result<Config, figment::Error> {
    let home = jail.directory().to_path_buf();
    jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
    jail.set_env("HOME", home.to_str().unwrap_or_default());
    config::load(None).map_err(|e| figment::Error::from(e.to_string()))
}

fn bind_for(addr: &str) -> anyhow::Result<Listener> {
    let ns = config::broker_abstract_name(addr)?;
    Ok(ListenerOptions::new().name(ns).create_tokio()?)
}

/// Accepts one connection, decodes the request, replies with `resp`.
async fn serve_one(listener: Listener, resp: BrokerResponse) -> anyhow::Result<()> {
    let stream = listener.accept().await?;
    let codec = LengthDelimitedCodec::builder().max_frame_length(MAX_FRAME_LEN).new_codec();
    let mut framed = Framed::new(stream, codec);
    let frame = framed.next().await.ok_or_else(|| anyhow::anyhow!("no request frame"))??;
    let _req: ipc::BrokerRequest = serde_json::from_slice(&frame)?;
    let bytes = Bytes::from(serde_json::to_vec(&resp)?);
    framed.send(bytes).await?;
    Ok(())
}

fn status_info(persistent: bool) -> BrokerResponse {
    BrokerResponse::StatusInfo {
        state: SessionState::Active,
        device: "irrelevant".into(),
        persistent,
    }
}

/// Runs `fut` on a fresh runtime and translates its `anyhow::Error` into a `figment::Error` —
/// `Jail::expect_with`'s closure isn't async and its `Ok`/`Err` type is fixed to `figment`'s.
#[allow(clippy::result_large_err)]
fn run<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> Result<(), figment::Error> {
    tokio::runtime::Runtime::new()
        .map_err(|e| figment::Error::from(e.to_string()))?
        .block_on(fut)
        .map_err(|e| figment::Error::from(e.to_string()))
}

#[test]
fn classify_maps_persistent_to_already_running() {
    assert_eq!(classify(Some(true)), ProvisionState::AlreadyRunning);
}

#[test]
fn classify_maps_ephemeral_to_conflict() {
    assert_eq!(classify(Some(false)), ProvisionState::EphemeralConflict);
}

#[test]
fn classify_maps_unreachable_to_unreachable() {
    assert_eq!(classify(None), ProvisionState::Unreachable);
}

#[test]
#[serial]
fn ensure_running_is_ok_when_daemon_already_running() {
    figment::Jail::expect_with(|jail| {
        let cfg = load_test_config(jail)?;
        run(async move {
            let addr = "TE:ST:00:00:03:01";
            let listener = bind_for(addr)?;
            let server = tokio::spawn(serve_one(listener, status_info(true)));

            ensure_running(&cfg, Some(addr), None).await?;

            server.await??;
            Ok(())
        })
    });
}

#[test]
#[serial]
fn ensure_running_errors_on_ephemeral_conflict() {
    figment::Jail::expect_with(|jail| {
        let cfg = load_test_config(jail)?;
        run(async move {
            let addr = "TE:ST:00:00:03:02";
            let listener = bind_for(addr)?;
            let server = tokio::spawn(serve_one(listener, status_info(false)));

            let Err(err) = ensure_running(&cfg, Some(addr), None).await else {
                anyhow::bail!("expected an EphemeralConflict error");
            };
            anyhow::ensure!(
                matches!(&err, ProvisionError::EphemeralConflict(a) if a == addr),
                "unexpected error: {err}"
            );

            server.await??;
            Ok(())
        })
    });
}
