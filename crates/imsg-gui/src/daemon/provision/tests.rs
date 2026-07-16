//! `classify` and `classify_launch` are pure and tested directly. `ensure_running`'s
//! `AlreadyRunning`/`EphemeralConflict` branches return before ever spawning, so they're
//! real-socket tested the same way as `daemon/tests.rs`'s `status`/`stop` tests — real `Config`
//! via `figment::Jail` (same isolation approach `config/tests.rs` uses, `#[serial]` since `Jail`
//! mutates process-global env vars). The `Unreachable` (spawn) branch isn't — the spawn itself is
//! now `imsg_proc::respawn_self`, tested there.

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
///
/// Returns `config`'s own (small, already-boxed) error rather than `figment::Error` —
/// `Jail::expect_with`'s closure return type is fixed to the latter and it's large by figment's
/// own design, so callers convert at that boundary instead of this function carrying it.
fn load_test_config(jail: &mut figment::Jail) -> Result<Config, config::ConfigError> {
    let home = jail.directory().to_path_buf();
    jail.set_env("IMSG_DEVICE__ADDRESS", "AA:BB:CC:DD:EE:FF");
    jail.set_env("HOME", home.to_str().unwrap_or_default());
    config::load(None)
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

fn session_status(state: SessionState) -> BrokerResponse {
    BrokerResponse::StatusInfo { state, device: "irrelevant".into(), persistent: true }
}

/// Runs `fut` on a fresh runtime — `Jail::expect_with`'s closure isn't async, so tests need
/// their own runtime to drive one. Callers convert the `anyhow::Error` into the closure's
/// required `figment::Error` themselves (see `load_test_config`'s doc for why this stays out of
/// the signature here).
fn run<F: std::future::Future<Output = anyhow::Result<()>>>(fut: F) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(fut)
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
        let cfg = load_test_config(jail).map_err(|e| figment::Error::from(e.to_string()))?;
        run(async move {
            let addr = "TE:ST:00:00:03:01";
            let listener = bind_for(addr)?;
            let server = tokio::spawn(serve_one(listener, status_info(true)));

            ensure_running(&cfg, Some(addr), None).await?;

            server.await??;
            Ok(())
        })
        .map_err(|e| figment::Error::from(e.to_string()))
    });
}

#[test]
#[serial]
fn ensure_running_errors_on_ephemeral_conflict() {
    figment::Jail::expect_with(|jail| {
        let cfg = load_test_config(jail).map_err(|e| figment::Error::from(e.to_string()))?;
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
        .map_err(|e| figment::Error::from(e.to_string()))
    });
}

#[test]
fn announce_when_connected_returns_once_session_is_active() -> anyhow::Result<()> {
    run(async {
        let addr = "TE:ST:00:00:04:01";
        let listener = bind_for(addr)?;
        let server = tokio::spawn(serve_one(listener, session_status(SessionState::Active)));

        tokio::time::timeout(
            std::time::Duration::from_secs(15),
            announce_when_connected(addr.to_owned()),
        )
        .await
        .map_err(|_| anyhow::anyhow!("announce_when_connected never returned"))?;

        server.await??;
        Ok(())
    })
}

#[test]
fn announce_when_connected_gives_up_on_failed_session() -> anyhow::Result<()> {
    run(async {
        let addr = "TE:ST:00:00:04:02";
        let listener = bind_for(addr)?;
        let server = tokio::spawn(serve_one(listener, session_status(SessionState::Failed)));

        tokio::time::timeout(
            std::time::Duration::from_secs(15),
            announce_when_connected(addr.to_owned()),
        )
        .await
        .map_err(|_| anyhow::anyhow!("announce_when_connected never returned"))?;

        server.await??;
        Ok(())
    })
}

#[test]
fn classify_launch_returns_gui_when_headless_arg_absent() {
    let args: Vec<String> = vec!["imsg-gui".to_owned()];
    assert_eq!(classify_launch(&args), Ok(Launch::Gui));
}

#[test]
fn classify_launch_extracts_device_and_config() {
    let args: Vec<String> =
        ["imsg-gui", HEADLESS_ARG, "--device", "AA:BB:CC:DD:EE:FF", "--config", "/tmp/imsg.toml"]
            .into_iter()
            .map(str::to_owned)
            .collect();
    assert_eq!(
        classify_launch(&args),
        Ok(Launch::Headless(HeadlessArgs {
            device: "AA:BB:CC:DD:EE:FF".to_owned(),
            config_path: Some("/tmp/imsg.toml".into()),
        }))
    );
}

#[test]
fn classify_launch_extracts_device_without_config() {
    let args: Vec<String> = ["imsg-gui", HEADLESS_ARG, "--device", "AA:BB:CC:DD:EE:FF"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert_eq!(
        classify_launch(&args),
        Ok(Launch::Headless(HeadlessArgs {
            device: "AA:BB:CC:DD:EE:FF".to_owned(),
            config_path: None,
        }))
    );
}

#[test]
fn classify_launch_errors_when_device_missing() {
    let args: Vec<String> = ["imsg-gui", HEADLESS_ARG].into_iter().map(str::to_owned).collect();
    assert_eq!(classify_launch(&args), Err(HeadlessArgsError::MissingDevice));
}
