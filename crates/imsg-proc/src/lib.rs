//! Self-respawn: detach the current binary as a background child.
//!
//! One shared spawn/log-redirect/detach primitive for every "re-invoke myself as a background
//! process" call site in the workspace — the CLI's backgrounded `daemon start`, the CLI's
//! ephemeral one-shot broker, and the GUI's headless self-provisioned daemon — previously three
//! independent, near-identical copies differing only in argv verb and stdio/detach behavior.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::fs::OpenOptions;
use tokio::process::{Child, Command};

/// Builds the argv for re-invoking the current binary: `verb` followed by `--device <addr>`
/// and, if set, `--config <config_path>`.
///
/// # Errors
///
/// Returns an error if `config_path` is set and isn't valid UTF-8.
fn respawn_args(verb: &[&str], addr: &str, config_path: Option<&Path>) -> Result<Vec<String>> {
    let mut args: Vec<String> = verb.iter().map(|s| (*s).to_owned()).collect();
    args.push("--device".to_owned());
    args.push(addr.to_owned());
    if let Some(p) = config_path {
        args.push("--config".to_owned());
        args.push(p.to_str().context("config path is not valid UTF-8")?.to_owned());
    }
    Ok(args)
}

/// Opens (creating/truncating) `log_path` for a detached child's stdio, creating its parent
/// directory first. `0o600` on Unix — these logs outlive this process and may carry message
/// content, so keep them off-limits to other users.
async fn open_log(log_path: &Path) -> Result<std::fs::File> {
    if let Some(parent) = log_path.parent() {
        tokio::fs::create_dir_all(parent).await.context("creating log directory")?;
    }
    let mut open_opts = OpenOptions::new();
    open_opts.create(true).write(true).truncate(true);
    #[cfg(unix)]
    open_opts.mode(0o600);
    Ok(open_opts
        .open(log_path)
        .await
        .with_context(|| format!("opening log file: {}", log_path.display()))?
        .into_std()
        .await)
}

/// Re-execs `current_exe()` with `verb --device <addr> [--config <config_path>]`.
///
/// Stdio redirected to `log_path`. Stdout is discarded unless `capture_stdout` is set (stderr
/// always goes to the log); if `detach` is set, the child is moved into its own process group
/// (Unix) so it survives this process exiting.
///
/// # Errors
///
/// Returns an error if the current executable path can't be resolved, `config_path` isn't
/// valid UTF-8, the log file can't be created/opened, or spawning fails.
pub async fn respawn_self(
    verb: &[&str],
    addr: &str,
    config_path: Option<&Path>,
    log_path: &Path,
    capture_stdout: bool,
    detach: bool,
) -> Result<Child> {
    let args = respawn_args(verb, addr, config_path)?;
    let log_file = open_log(log_path).await?;
    let exe = std::env::current_exe().context("resolving current executable path")?;

    let mut cmd = Command::new(exe);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(if capture_stdout {
        Stdio::from(log_file.try_clone().context("duplicating log file handle")?)
    } else {
        Stdio::null()
    });
    cmd.stderr(Stdio::from(log_file));
    #[cfg(unix)]
    if detach {
        cmd.process_group(0);
    }
    cmd.spawn().context("spawning detached subprocess")
}

#[cfg(test)]
mod tests;
