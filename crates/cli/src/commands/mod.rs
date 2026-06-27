//! Subcommand handlers and the top-level dispatch.

pub mod broker;
pub mod config;
pub mod conn;
pub mod contacts;
pub mod delete;
pub mod folders;
pub mod get;
pub mod hub;
pub mod list;
pub mod send;
pub mod spoke;
pub mod sync;
pub mod threads;
pub mod unsync;
pub mod watch;
pub mod watch_hub;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::cli::{BrokerCmd, Cli, Command, ConfigCmd, SpokeCmd};
use crate::output;
use crate::progress::with_spinner;

/// Implements all commands: `config`, `list`, `folders`, `get`, `delete`, `send`,
/// `contacts`, `threads`, `watch`, `sync`, `unsync`, `hub`, and `spoke`. All network-bound one-shot commands
/// run under a [`with_spinner`] progress indicator; the result is printed via [`output::line`]
/// after the spinner clears. `watch` and `hub` are streaming/blocking — they manage their own
/// output and return `None` from [`run_command`].
///
/// Ensures the spoke [`transport::iroh::Endpoint`] is closed via
/// [`transport::iroh::Endpoint::close`] on all exit paths — success, error, and early return —
/// by delegating command execution to [`run_command`] before touching the endpoint.
///
/// # Errors
///
/// Returns an error if the selected handler fails or the output cannot be written.
pub(crate) async fn dispatch(cli: Cli) -> Result<()> {
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

/// Executes the selected command and returns its printable output, or `None` for streaming
/// commands (`hub`, `watch`) that manage their own output.
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
        Command::Config { cmd } => Some(run_config(cmd, config_path)?),
        Command::Hub => {
            hub::run(&load(config_path)?).await?;
            None
        }
        Command::Spoke { cmd } => Some(match cmd {
            SpokeCmd::Add { key } => spoke::run_add(&key).await?,
        }),
        Command::Broker { cmd } => Some(match cmd {
            BrokerCmd::Status => broker::run_status(&load(config_path)?, device).await?,
        }),
        Command::List { folder, unread, long, from, since, limit, offset } => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            let opts = list::ListOpts { folder, unread, from, since, limit, offset, long };
            Some(run_list(&cfg, spoke, device, opts, &db, bpath.as_deref()).await?)
        }
        Command::Folders => {
            Some(with_spinner("folders", folders::run(&load(config_path)?, spoke, device)).await?)
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
        Command::Contacts { list, get, lookup, path, raw, limit, page } => {
            let cfg = load(config_path)?;
            let opts = contacts::ContactsOpts { list, get, lookup, path, raw, limit, page };
            Some(with_spinner("contacts", contacts::run(&cfg, spoke, device, opts)).await?)
        }
        Command::Threads => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            Some(run_threads(&cfg, spoke, device, &db, bpath.as_deref()).await?)
        }
        Command::Watch => {
            let (cfg, db, bpath) = load_with_store(config_path).await?;
            watch::run(&cfg, spoke, device, bpath.as_deref(), &db).await?;
            None
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

/// Executes a `config` subcommand — no network I/O, no store access.
fn run_config(cmd: ConfigCmd, config_path: Option<PathBuf>) -> Result<String> {
    match cmd {
        ConfigCmd::Show => config::run_show(config_path),
        ConfigCmd::SetDevice { address } => config::run_set_device(&address),
    }
}

/// Dispatches `list` to the local store or phone based on opt-in state.
async fn run_list(
    cfg: &::config::Config,
    spoke: Option<&transport::iroh::Endpoint>,
    device: Option<&str>,
    opts: list::ListOpts,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        with_spinner("listing", list::run_store(opts, db)).await
    } else {
        with_spinner("listing", list::run(cfg, spoke, device, opts, config_path)).await
    }
}

/// Dispatches `get` to the local store or phone based on opt-in state.
async fn run_get(
    cfg: &::config::Config,
    spoke: Option<&transport::iroh::Endpoint>,
    device: Option<&str>,
    handle: String,
    mark_read: bool,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        with_spinner("fetching", get::run_store(handle, mark_read, db)).await
    } else {
        with_spinner("fetching", get::run(cfg, spoke, device, handle, mark_read, config_path)).await
    }
}

/// Dispatches `threads` to the local store or phone based on opt-in state.
async fn run_threads(
    cfg: &::config::Config,
    spoke: Option<&transport::iroh::Endpoint>,
    device: Option<&str>,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        with_spinner("threads", threads::run_store(db)).await
    } else {
        with_spinner("threads", threads::run(cfg, spoke, device, config_path)).await
    }
}

/// Dispatches `send` to the store-backed outbox or a device-only push based on opt-in state.
///
/// Opted-in goes through `session::outbox::send_sms` (enqueue + push + reconcile); not opted-in
/// pushes to the device only, leaving no local outbox row.
async fn run_send(
    cfg: &::config::Config,
    spoke: Option<&transport::iroh::Endpoint>,
    device: Option<&str>,
    number: String,
    message: String,
    db: &store::Store,
    config_path: Option<&Path>,
) -> Result<String> {
    if is_opted_in(db).await {
        let fut = send::run(cfg, spoke, device, number, message, db, config_path);
        with_spinner("sending", fut).await
    } else {
        let fut = send::run_live(cfg, spoke, device, number, message, config_path);
        with_spinner("sending", fut).await
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

/// Returns `true` when `sync_enabled = "true"` is set in the store `meta` table.
///
/// Any store error is treated as not opted in so the caller falls back to the phone path;
/// a warning is emitted so the failure is visible in logs.
async fn is_opted_in(store: &store::Store) -> bool {
    match store.get_meta("sync_enabled").await {
        Ok(v) => v.as_deref() == Some("true"),
        Err(e) => {
            tracing::warn!("failed to read sync_enabled from store, falling back to phone: {e}");
            false
        }
    }
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
fn load(path: Option<PathBuf>) -> Result<::config::Config> {
    ::config::load(path)
        .context("loading config (run `imsg config set-device <ADDR>` if device.address is unset)")
}

/// Initialises the keyring and opens the encrypted message store.
///
/// `cfg.store.resolve()` provides the DB path; falls back to the XDG data dir default.
/// Must be called after [`load`]. Errors if the Secret Service D-Bus daemon is unavailable,
/// the keyring entry is corrupted, or the DB file cannot be opened/migrated.
///
/// # Errors
///
/// Returns an error if the data directory is unavailable, keyring init fails, key retrieval
/// fails, or [`store::Store::open`] fails.
async fn open_store(cfg: &::config::Config) -> Result<store::Store> {
    let path =
        cfg.store.resolve().context("no data directory available (set HOME or XDG_DATA_HOME)")?;
    let ready = keyring::init_store().context("Secret Service store init failed")?;
    let key = keyring::get_or_create_db_key(&ready).context("getting database encryption key")?;
    store::Store::open(path, key).await.context("opening message store")
}
