//! `daemon` subcommand — runs the broker as a persistent, long-lived process instead of the
//! ephemeral one-shot broker other commands spawn.
//!
//! `start` detaches into the background by default: it re-execs `imsg daemon start
//! --foreground` as a child in its own process group with stdio redirected to a log file, then
//! returns once that child's socket is reachable. `--foreground` is what the detached child
//! actually runs (and is also directly usable under a process supervisor, e.g. a systemd
//! unit's `ExecStart`, which needs the main process to stay attached). `stop`/`status` talk to
//! an already-running daemon over the same IPC socket every other command uses — no process
//! management involved, and neither auto-starts it.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use config::Config;
use ipc::SessionState;
use store::Store;

use crate::cli::DaemonCmd;
use crate::commands::{broker, load, open_store};
use crate::output;

/// Dispatches `daemon` subcommands.
///
/// # Errors
///
/// See [`start_foreground`], [`start_background`], [`stop`], [`install`], [`uninstall`], and
/// [`broker::run_status`][crate::commands::broker::run_status].
pub(crate) async fn dispatch(
    cmd: DaemonCmd,
    device: Option<&str>,
    config_path: Option<PathBuf>,
) -> Result<Option<String>> {
    match cmd {
        DaemonCmd::Start { foreground: true } => {
            let cfg = load(config_path)?;
            let db = open_store(&cfg).await?;
            start_foreground(cfg, device.map(str::to_owned), db).await?;
            Ok(None)
        }
        DaemonCmd::Start { foreground: false } => {
            let cfg = load(config_path.clone())?;
            Ok(Some(start_background(&cfg, device, config_path).await?))
        }
        DaemonCmd::Stop => {
            let cfg = load(config_path)?;
            Ok(Some(stop(&cfg, device).await?))
        }
        DaemonCmd::Status => {
            let cfg = load(config_path)?;
            Ok(Some(broker::run_status(&cfg, device, "daemon").await?))
        }
        DaemonCmd::Install { system } => {
            let addr = match device {
                Some(d) => d.to_owned(),
                None => default_device(config_path.clone(), system)?,
            };
            Ok(Some(install(&addr, config_path.as_deref(), system)?))
        }
        DaemonCmd::Uninstall { system } => Ok(Some(uninstall(system)?)),
    }
}

/// Resolves the default `--device` for `daemon install` when it isn't passed explicitly.
///
/// Under `--system`, loading via this process's own `$HOME` finds nothing useful: plain `sudo`
/// resets `HOME` to `/root` (only `sudo -E` preserves it, which operators won't reliably
/// remember), so config lookup falls back to the real `sudo`-invoking user's `~/.config/imsg`
/// instead of root's — the same directory [`service::install`] points the unit's own
/// `XDG_CONFIG_HOME` at, so this stays consistent with what the running service will read.
fn default_device(config_path: Option<PathBuf>, system: bool) -> Result<String> {
    if system && config_path.is_none() {
        let home = service::invoking_home().context("resolving config for --system install")?;
        let explicit = home.join(".config/imsg/imsg.toml");
        return Ok(load(Some(explicit))?.device.address().to_owned());
    }
    Ok(load(config_path)?.device.address().to_owned())
}

/// Runs the broker in this process until the shutdown coordinator converges — what the
/// detached child from [`start_background`] actually runs, and also directly usable under a
/// process supervisor. Sets `daemon_enabled = "true"` first — independent of `sync_enabled`:
/// this never opts the CLI's one-shot commands into local-store reads on its own, that stays
/// under `imsg sync`'s sole control.
///
/// # Errors
///
/// Returns an error if `daemon_enabled` cannot be persisted or `run_daemon` fails.
async fn start_foreground(cfg: Config, device: Option<String>, store: Store) -> Result<()> {
    store.set_meta("daemon_enabled", "true").await?;
    output::line("daemon starting — Ctrl-C to stop")?;
    let addr = device.clone().unwrap_or_else(|| cfg.device.address().to_owned());
    tokio::spawn(announce_when_connected(addr));
    imsg_broker::run_daemon(cfg, device, store).await
}

/// Polls the daemon's own IPC socket until the MAP session reports `Active`, then prints a
/// one-time confirmation — `run_daemon` blocks forever with no feedback of its own once
/// serving starts, so without this a healthy foreground daemon looks stuck on "starting".
async fn announce_when_connected(addr: String) {
    loop {
        match broker::query_state(&addr).await {
            Some(SessionState::Active) => {
                let _ = output::line(&format!("daemon for {addr}: connected"));
                return;
            }
            Some(SessionState::Failed) => return,
            _ => {}
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Spawns a detached child (own process group, log-file stdio) and waits for its socket to
/// become reachable. Idempotent: returns immediately if already running.
///
/// # Errors
///
/// Returns an error if the log file/child cannot be created or it never becomes reachable.
async fn start_background(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<PathBuf>,
) -> Result<String> {
    let addr = device.unwrap_or_else(|| cfg.device.address()).to_owned();
    match broker::query_persistent(&addr).await {
        Some(true) => return Ok(format!("daemon for {addr}: already running")),
        Some(false) => {
            return Err(anyhow::anyhow!(
                "an ephemeral broker for {addr} is already using this socket — wait for it to \
                 idle out (or check `imsg broker status`) before starting the daemon"
            ));
        }
        None => {}
    }

    let log_path = config::daemon_log_path(&addr);
    let mut child = imsg_proc::respawn_self(
        &["daemon", "start", "--foreground"],
        &addr,
        config_path.as_deref(),
        &log_path,
        true,
        true,
    )
    .await?;
    broker::connect_retry(
        &addr,
        &mut child,
        &log_path,
        cfg.broker.readiness_wait(),
        cfg.broker.readiness_poll(),
    )
    .await?;
    Ok(format!("daemon for {addr}: started (log: {})", log_path.display()))
}

/// Sends a graceful `Shutdown` request; a no-op if nothing is running.
///
/// # Errors
///
/// Returns an error if the connection succeeds but the request/response fails.
async fn stop(cfg: &Config, device: Option<&str>) -> Result<String> {
    broker::run_stop(cfg, device).await
}

/// Maps `--system` to the matching [`service::ServiceLevel`]; the default is per-user.
const fn level(system: bool) -> service::ServiceLevel {
    if system {
        service::ServiceLevel::System
    } else {
        service::ServiceLevel::User
    }
}

/// Registers the daemon with the native OS service manager via [`service::install`], so it
/// starts on boot/login and restarts on failure. Always bakes `--device <addr>` into the
/// service's `imsg daemon start --foreground` invocation (resolved from config if not passed
/// explicitly) — a `--system` service typically runs as root, which has no user config to fall
/// back on at its own start time, so the address can't be left to runtime lookup. `--config` is
/// only forwarded if given explicitly.
///
/// # Errors
///
/// Returns an error if no native service manager is available or it rejects the install.
fn install(addr: &str, config_path: Option<&Path>, system: bool) -> Result<String> {
    let lvl = level(system);
    service::install(Some(addr), config_path, lvl).context("installing daemon service")?;
    Ok(format!("daemon service installed ({lvl:?})"))
}

/// Unregisters the daemon service via [`service::uninstall`], reporting whether anything was
/// there to remove.
///
/// # Errors
///
/// Returns an error if no native service manager is available or it rejects the uninstall.
fn uninstall(system: bool) -> Result<String> {
    let lvl = level(system);
    let outcome = service::uninstall(lvl).context("uninstalling daemon service")?;
    Ok(uninstall_message(outcome, lvl))
}

/// Renders [`service::uninstall`]'s outcome, keeping a no-op from reading as a removal.
fn uninstall_message(outcome: service::UninstallOutcome, lvl: service::ServiceLevel) -> String {
    match outcome {
        service::UninstallOutcome::Uninstalled => format!("daemon service uninstalled ({lvl:?})"),
        service::UninstallOutcome::NotInstalled => {
            format!("no daemon service installed ({lvl:?})")
        }
    }
}

#[cfg(test)]
mod tests;
