//! Private broker runtime boundary: socket bind/serve, IPC handling, the device actor and its
//! lifecycle, and MAP dispatch. Nothing here escapes the crate — [`crate::run`] is the only
//! public entry point.

mod actor;
mod daemon;
mod handler;
mod server;
mod shutdown;
mod types;

use anyhow::{Context, Result};
use config::Config;
use interprocess::local_socket::tokio::Listener as IpcListener;
use store::Store;

pub use daemon::run_daemon;

/// Binds the abstract broker socket for `addr` and serves IPC requests until idle timeout or
/// fatal MAP failure.
///
/// The device actor establishes the MAP session lazily, so the socket is reachable before the
/// (slow, fallible) Bluetooth connection completes.
///
/// If another broker process already holds the abstract name for this `addr`, this
/// function calls `process::exit(0)` — the kernel's `EADDRINUSE` is the single-instance
/// election; no file cleanup is needed on any exit path.
///
/// `store` is an already-opened message store supplied by the CLI entry point.
/// `device_override` replaces `cfg.device.address()` as the RFCOMM target.
///
/// # Errors
///
/// Returns an error if socket binding or the accept loop fails fatally. A failed MAP connect is
/// handled by the actor (it shuts the broker down), not returned here.
pub async fn run(cfg: Config, device_override: Option<String>, store: Store) -> Result<()> {
    let (addr_str, addr, map_channel, listener) = bind(&cfg, device_override.as_deref(), true)?;
    server::serve(cfg, addr_str, addr, map_channel, store, listener).await
}

/// Parses the target device address and wins (or exits/errors on) the abstract-socket election.
///
/// Shared by [`run`] and [`daemon::run_daemon`] — the only difference between ephemeral and
/// persistent mode is which `server::serve*` variant runs on the result, plus `allow_silent_exit`
/// (see [`server::bind_or_exit`]): ephemeral mode exits 0 on a lost race, persistent mode errors.
fn bind(
    cfg: &Config,
    device_override: Option<&str>,
    allow_silent_exit: bool,
) -> Result<(String, bluer::Address, u8, IpcListener)> {
    let addr_str = device_override.unwrap_or_else(|| cfg.device.address()).to_owned();
    let map_channel = cfg.device.map_channel;
    let addr: bluer::Address =
        addr_str.parse().with_context(|| format!("invalid device address: {addr_str}"))?;

    let listener = server::bind_or_exit(&addr_str, allow_silent_exit)?;

    Ok((addr_str, addr, map_channel, listener))
}
