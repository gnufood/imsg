//! Subcommand handlers and the top-level dispatch.

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
pub mod threads;
pub mod watch;
pub mod watch_hub;

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli::{Cli, Command, ConfigCmd, SpokeCmd};
use crate::output;
use crate::progress::with_spinner;

/// Implements all commands: `config`, `list`, `folders`, `get`, `delete`, `send`,
/// `contacts`, `threads`, `watch`, `hub`, and `spoke`. All network-bound one-shot commands
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
        Command::Config { cmd } => Some(match cmd {
            ConfigCmd::Show => config::run_show(config_path)?,
            ConfigCmd::SetDevice { address } => config::run_set_device(&address)?,
        }),
        Command::Hub => {
            let cfg = load(config_path)?;
            hub::run(&cfg).await?;
            None
        }
        Command::Spoke { cmd } => Some(match cmd {
            SpokeCmd::Add { key } => spoke::run_add(&key).await?,
        }),
        Command::List { folder, unread, long, from, since, limit, offset } => {
            let cfg = load(config_path)?;
            let filter = list::build_filter(unread, from, since, limit, offset);
            Some(
                with_spinner("listing", list::run(&cfg, spoke, device, folder, filter, long))
                    .await?,
            )
        }
        Command::Folders => {
            let cfg = load(config_path)?;
            Some(with_spinner("folders", folders::run(&cfg, spoke, device)).await?)
        }
        Command::Get { handle, folder, mark_read } => {
            let cfg = load(config_path)?;
            Some(
                with_spinner("fetching", get::run(&cfg, spoke, device, handle, folder, mark_read))
                    .await?,
            )
        }
        Command::Delete { handle, folder, undelete } => {
            let cfg = load(config_path)?;
            Some(
                with_spinner(
                    "deleting",
                    delete::run(&cfg, spoke, device, handle, folder, undelete),
                )
                .await?,
            )
        }
        Command::Send { number, message } => {
            let cfg = load(config_path)?;
            Some(with_spinner("sending", send::run(&cfg, spoke, device, number, message)).await?)
        }
        Command::Contacts { list, get, lookup, path, raw, limit, page } => {
            let cfg = load(config_path)?;
            Some(
                with_spinner(
                    "contacts",
                    contacts::run(
                        &cfg,
                        spoke,
                        device,
                        contacts::ContactsOpts { list, get, lookup, path, raw, limit, page },
                    ),
                )
                .await?,
            )
        }
        Command::Threads => {
            let cfg = load(config_path)?;
            Some(with_spinner("threads", threads::run(&cfg, spoke, device)).await?)
        }
        Command::Watch => {
            let cfg = load(config_path)?;
            watch::run(&cfg, spoke, device).await?;
            None
        }
    };
    Ok(out)
}

/// Loads the layered configuration, attaching a hint about the most common failure.
fn load(path: Option<PathBuf>) -> Result<::config::Config> {
    ::config::load(path)
        .context("loading config (run `imsg config set-device <ADDR>` if device.address is unset)")
}
