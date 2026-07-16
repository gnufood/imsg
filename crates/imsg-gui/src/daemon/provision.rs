//! Self-provisioning: the GUI has no mode where it runs without a reachable daemon.
//!
//! Unlike the CLI it can't shell out to a separate `imsg` binary to become one (bundling it
//! would defeat the point of extracting `imsg-broker`/`imsg-broker-client` as CLI-agnostic in
//! the first place). Instead `imsg-gui` re-execs *itself* into a hidden headless mode that runs
//! [`imsg_broker::run_daemon`] in-process, via [`imsg_proc::respawn_self`] — the same shared
//! spawn/detach primitive the CLI's `daemon.rs::start_background`/`start_foreground` use, with
//! `imsg-gui` as both the launcher and the payload.

use std::path::PathBuf;

use config::Config;
use store::Store;

/// Tells a re-exec'd `imsg-gui` child to run headlessly as the daemon instead of opening a
/// window.
///
/// Never user-facing — only [`ensure_running`]'s spawn ever sets it; `main.rs`'s startup is the
/// only place that ever needs to check for it.
pub const HEADLESS_ARG: &str = "--__daemon_foreground";

/// What a [`broker_client::query_persistent`] check implies about whether to self-provision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisionState {
    /// A persistent daemon already answers at the target address — nothing to spawn.
    AlreadyRunning,
    /// Something is listening, but as the ephemeral one-shot broker, not a daemon — refuses to
    /// spawn onto the same socket while it's in use rather than race it.
    EphemeralConflict,
    /// Nothing reachable — safe to spawn.
    Unreachable,
}

/// Classifies a [`broker_client::query_persistent`] result into a spawn decision. Pure and
/// deterministic, kept separate from the reachability check itself so the branching is testable
/// without a socket.
#[must_use]
pub const fn classify(persistent: Option<bool>) -> ProvisionState {
    match persistent {
        Some(true) => ProvisionState::AlreadyRunning,
        Some(false) => ProvisionState::EphemeralConflict,
        None => ProvisionState::Unreachable,
    }
}

/// Failure self-provisioning a daemon.
#[derive(Debug, thiserror::Error)]
pub enum ProvisionError {
    /// Something is listening at the target address, but as an ephemeral broker, not a daemon.
    #[error("an ephemeral broker for {0} is already using this socket")]
    EphemeralConflict(String),
    /// Spawning the detached headless child, waiting for its socket to become reachable, or (in
    /// the headless child itself) running the daemon, failed.
    #[error("{0}")]
    Broker(#[from] anyhow::Error),
    /// The headless child couldn't record that the daemon opt-in ran.
    #[error("recording daemon_enabled: {0}")]
    Meta(#[from] store::Error),
}

/// Ensures a daemon is reachable for `device` (or `cfg`'s default), self-provisioning one if
/// needed.
///
/// Spawns a detached headless `imsg-gui` child if nothing answers yet, and returns once the
/// socket is confirmed reachable. Idempotent: returns immediately if a daemon is already
/// running.
///
/// # Errors
///
/// Returns [`ProvisionError::EphemeralConflict`] if the address is held by an ephemeral broker
/// instead of a daemon, or [`ProvisionError::Broker`] if spawning fails or the socket never
/// becomes reachable within `cfg.broker`'s readiness budget.
pub async fn ensure_running(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<PathBuf>,
) -> Result<(), ProvisionError> {
    let addr = device.unwrap_or_else(|| cfg.device.address()).to_owned();
    match classify(broker_client::query_persistent(&addr).await) {
        ProvisionState::AlreadyRunning => return Ok(()),
        ProvisionState::EphemeralConflict => {
            return Err(ProvisionError::EphemeralConflict(addr));
        }
        ProvisionState::Unreachable => {}
    }

    let log_path = config::daemon_log_path(&addr);
    let mut child = imsg_proc::respawn_self(
        &[HEADLESS_ARG],
        &addr,
        config_path.as_deref(),
        &log_path,
        true,
        true,
    )
    .await?;
    broker_client::connect_retry(
        &addr,
        &mut child,
        &log_path,
        cfg.broker.readiness_wait(),
        cfg.broker.readiness_poll(),
    )
    .await?;
    Ok(())
}

/// Runs the broker in this process until shutdown.
///
/// What a self-provisioned headless child (spawned by [`ensure_running`], recognized via
/// [`HEADLESS_ARG`]) actually runs. Mirrors the CLI's `daemon.rs::start_foreground`, minus its
/// terminal output (no terminal exists here). Not unit-tested: same "requires a real
/// MAP/Bluetooth session" reason `start_foreground` has no tests of its own — `run_daemon`'s
/// behavior is covered by `imsg-broker`'s own test suite.
///
/// # Errors
///
/// Returns [`ProvisionError::Meta`] if `daemon_enabled` can't be persisted, or
/// [`ProvisionError::Broker`] if `run_daemon` fails.
pub async fn run_headless(
    cfg: Config,
    device: Option<String>,
    store: Store,
) -> Result<(), ProvisionError> {
    store.set_meta("daemon_enabled", "true").await?;
    let addr = device.clone().unwrap_or_else(|| cfg.device.address().to_owned());
    tauri::async_runtime::spawn(announce_when_connected(addr));
    imsg_broker::run_daemon(cfg, device, store).await?;
    Ok(())
}

/// Polls the daemon's own IPC socket until the MAP session reports `Active`, then logs a
/// one-time confirmation — `run_daemon` blocks forever with no feedback of its own once serving
/// starts. Mirrors the CLI daemon's `announce_when_connected` so a self-provisioned daemon's
/// captured log records the same "connected" line a CLI-started daemon's does.
async fn announce_when_connected(addr: String) {
    loop {
        match broker_client::query_state(&addr).await {
            Some(ipc::SessionState::Active) => {
                tracing::info!("daemon for {addr}: connected");
                return;
            }
            Some(ipc::SessionState::Failed) => return,
            _ => {}
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

/// What `main.rs` should do based on the process's argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// Normal GUI launch — [`HEADLESS_ARG`] wasn't present.
    Gui,
    /// A self-respawned headless child (see [`ensure_running`]'s spawn).
    Headless(HeadlessArgs),
}

/// `--device`/`--config` values extracted from a headless launch's argv — the inverse of
/// `imsg_proc`'s `respawn_args`' `[HEADLESS_ARG, "--device", addr, ("--config", path)?]` shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessArgs {
    /// The device address `ensure_running`'s spawn passed via `--device`.
    pub device: String,
    /// The `--config` path, if one was passed.
    pub config_path: Option<PathBuf>,
}

/// [`classify_launch`] couldn't make sense of a headless invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HeadlessArgsError {
    /// [`HEADLESS_ARG`] was present without a `--device` value — `respawn_args`'s contract
    /// broken; should never happen outside a bug in this crate, since only our own spawn ever
    /// sets [`HEADLESS_ARG`].
    #[error("headless launch missing --device")]
    MissingDevice,
}

/// Classifies process argv (`std::env::args()`, element 0 the executable path) as a normal GUI
/// launch or a self-respawned headless one, extracting `--device`/`--config` in the latter case.
///
/// # Errors
///
/// Returns [`HeadlessArgsError::MissingDevice`] if [`HEADLESS_ARG`] is present without
/// `--device`.
pub fn classify_launch(args: &[String]) -> Result<Launch, HeadlessArgsError> {
    if !args.iter().any(|a| a == HEADLESS_ARG) {
        return Ok(Launch::Gui);
    }
    let device = flag_value(args, "--device").ok_or(HeadlessArgsError::MissingDevice)?;
    let config_path = flag_value(args, "--config").map(PathBuf::from);
    Ok(Launch::Headless(HeadlessArgs { device, config_path }))
}

/// Returns the value following the first occurrence of `flag` in `args`, if any.
fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| i.checked_add(1))
        .and_then(|i| args.get(i))
        .cloned()
}

#[cfg(test)]
mod tests;
