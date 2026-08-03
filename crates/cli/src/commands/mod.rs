//! Subcommand handlers and the top-level dispatch.

pub mod broker;
pub mod config;
pub mod conn;
pub mod contacts;
pub mod daemon;
pub mod delete;
mod dispatch;
pub mod folders;
pub mod get;
pub mod hub;
pub mod list;
pub mod send;
pub mod spoke;
pub mod sync;
pub mod threads;
pub mod unsync;

use dispatch::{run_contacts, run_get, run_list, run_send, run_threads};

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::CommandFactory as _;

use crate::cli::{BrokerCmd, Cli, Command, ConfigCmd, SpokeCmd};
use crate::output;
use crate::progress::with_spinner;

/// Implements all commands: `config`, `list`, `folders`, `get`, `delete`, `send`,
/// `contacts`, `threads`, `sync`, `unsync`, `hub`, `spoke`, `broker`, and `daemon`.
///
/// All network-bound one-shot commands run under a `with_spinner` progress indicator; the
/// result is printed via `output::line` after the spinner clears. `hub` is streaming/blocking
/// — it manages its own output and returns `None` from `run_command`.
///
/// Ensures the spoke [`transport::iroh::Endpoint`] is closed via
/// [`transport::iroh::Endpoint::close`] on all exit paths — success, error, and early return —
/// by delegating command execution to `run_command` before touching the endpoint.
///
/// # Errors
///
/// Returns an error if the selected handler fails or the output cannot be written.
///
/// `pub` (not `pub(crate)`) so `main.rs` — a separate crate from this lib target — can call it.
pub async fn dispatch(cli: Cli) -> Result<()> {
    let Cli { hub, device, config: config_path, command, .. } = cli;
    let spoke = if hub {
        Some(transport::iroh::bind_spoke().await.context("binding iroh spoke endpoint")?)
    } else {
        None
    };
    let result = run_command(command, spoke.as_ref(), device.as_deref(), config_path).await;
    if let Some(ep) = &spoke {
        ep.close().await;
    }
    (result?).map_or_else(|| Ok(()), |out| output::line(&out))
}

/// Executes the selected command and returns its printable output, or `None` for the
/// streaming `hub` command, which manages its own output.
///
/// All `?` propagation stays within this function so [`dispatch`] can close the spoke endpoint
/// unconditionally after this returns, regardless of success or failure.
///
/// # Errors
///
/// Propagates any error returned by the selected command handler.
async fn run_command(
    command: Command,
    spoke: Option<&transport::iroh::Endpoint>,
    device: Option<&str>,
    config_path: Option<PathBuf>,
) -> Result<Option<String>> {
    let out = match command {
        Command::Config { cmd } => Some(run_config(cmd, config_path).await?),
        Command::Hub => {
            hub::run(&load(config_path)?).await?;
            None
        }
        Command::Spoke { cmd } => Some(match cmd {
            SpokeCmd::Add { key } => spoke::run_add(&key).await?,
        }),
        Command::Broker { cmd } => Some(match cmd {
            BrokerCmd::Status => broker::run_status(&load(config_path)?, device, "broker").await?,
        }),
        Command::List { folder, unread, long, from, since, limit, offset } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            let opts = list::ListOpts { folder, unread, from, since, limit, offset, long };
            Some(run_list(&cfg, spoke, device, opts, &db, bpath.as_deref()).await?)
        }
        Command::Folders => {
            // Cloned before `load` consumes it: the broker path forwards `--config` to the
            // subprocess it may have to spawn.
            let bpath = config_path.clone();
            let cfg = load(config_path)?;
            let fut = folders::run(&cfg, spoke, device, bpath.as_deref());
            Some(with_spinner("folders", fut).await?)
        }
        Command::Get { handle, folder: _, mark_read } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            Some(run_get(&cfg, spoke, device, handle, mark_read, &db, bpath.as_deref()).await?)
        }
        Command::Delete { handle, folder, undelete } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            let opts = delete::DeleteOpts { handle, folder, undelete };
            let fut = delete::run(&cfg, spoke, device, opts, &db, bpath.as_deref());
            Some(with_spinner("deleting", fut).await?)
        }
        Command::Send { number, message } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            Some(run_send(&cfg, spoke, device, number, message, &db, bpath.as_deref()).await?)
        }
        Command::Contacts { list, get, lookup, sync, path, raw, limit, page } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            let opts = contacts::ContactsOpts { list, get, lookup, sync, path, raw, limit, page };
            Some(run_contacts(&cfg, spoke, device, opts, &db, bpath.as_deref()).await?)
        }
        Command::Threads => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            Some(run_threads(&cfg, spoke, device, &db, bpath.as_deref()).await?)
        }
        Command::Sync { folder } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            let fut = sync::run(&cfg, spoke, device, &db, folder, bpath.as_deref());
            Some(with_spinner("syncing", fut).await?)
        }
        Command::BrokerServe => {
            let cfg = load(config_path)?;
            let db = open_store(&cfg).await?;
            imsg_broker::run(cfg, device.map(str::to_owned), db).await?;
            None
        }
        Command::Unsync { purge } => Some(run_unsync(purge, config_path).await?),
        Command::Daemon { cmd } => daemon::dispatch(cmd, device, config_path).await?,
        Command::Completions { shell } => {
            let mut buf = Vec::new();
            clap_complete::generate(shell, &mut Cli::command(), "imsg", &mut buf);
            Some(String::from_utf8(buf).context("completion script was not valid UTF-8")?)
        }
    };
    Ok(out)
}

/// Loads layered config and opens the store, returning the original config path alongside.
///
/// The returned `Option<PathBuf>` is the untouched `config_path`, preserved so callers can
/// forward `--config` to the broker subprocess after `load` has consumed its own copy.
async fn load_with_store(
    config_path: Option<PathBuf>,
) -> Result<(::config::Config, store::Store, Option<PathBuf>)> {
    let bpath = config_path.clone();
    let cfg = load(config_path)?;
    let db = open_store(&cfg).await?;
    Ok((cfg, db, bpath))
}

/// Executes a `config` subcommand. `Setup` is the only variant that touches the network
/// (Bluetooth SDP); the rest are local-file-only.
async fn run_config(cmd: ConfigCmd, config_path: Option<PathBuf>) -> Result<String> {
    match cmd {
        ConfigCmd::Show => config::run_show(config_path),
        ConfigCmd::SetDevice { address } => config::run_set_device(&address),
        ConfigCmd::Setup => config::run_setup().await,
    }
}

/// Disables sync, deleting the database if `purge` is set.
async fn run_unsync(purge: bool, config_path: Option<PathBuf>) -> Result<String> {
    let cfg = load(config_path)?;
    let db_path = cfg.store.resolve().context("no data directory available")?;
    if purge {
        unsync::purge(db_path)?;
        return Ok("sync disabled; database deleted".to_owned());
    }
    let db = open_store(&cfg).await?;
    unsync::disable(&db).await?;
    Ok("sync disabled; database preserved (re-enable with imsg sync)".to_owned())
}

/// Appends the live-read footer, ensuring the body ends with a newline first.
///
/// Used by the non-opted-in read paths (`list`/`get`/`threads`) where output comes straight
/// from the device, in place of the store-path freshness footer.
pub(crate) fn live_footer(mut out: String) -> String {
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("(live from device)");
    out
}

/// Formats a freshness footer for store-path read command output.
///
/// Shows the timestamp of the last completed sync, or a prompt to run `imsg sync`
/// if the store has never been synced.
pub(crate) fn freshness_line(last_sync_at: Option<i64>) -> String {
    last_sync_at.map_or_else(
        || "(never synced \u{2014} run 'imsg sync' to populate the store)".to_owned(),
        |ms| {
            format!(
                "(store as of {} \u{2014} run 'imsg sync' to refresh)",
                session::sync::ms_to_display(ms)
            )
        },
    )
}

/// Loads the layered configuration, attaching a hint about the most common failure.
pub(in crate::commands) fn load(path: Option<PathBuf>) -> Result<::config::Config> {
    ::config::load(path)
        .context("loading config (run `imsg config set-device <ADDR>` if device.address is unset)")
}

/// Initialises the keyring and opens the encrypted message store.
///
/// `cfg.store.resolve()` provides the DB path; falls back to the XDG data dir default.
/// Must be called after [`load`].
///
/// # Errors
///
/// Returns an error if the data directory is unavailable, keyring init fails, key retrieval
/// fails, or [`store::Store::open`] fails.
pub(in crate::commands) async fn open_store(cfg: &::config::Config) -> Result<store::Store> {
    let path =
        cfg.store.resolve().context("no data directory available (set HOME or XDG_DATA_HOME)")?;
    let ready = keyring::init_store().context("Secret Service store init failed")?;
    let key = keyring::get_or_create_db_key(&ready).context("getting database encryption key")?;
    store::Store::open(path, key).await.context("opening message store")
}
