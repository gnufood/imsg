//! imsg CLI — entry point for the iMessage/MAP client over Bluetooth MAP/PBAP.

// `pub mod` (not `mod`) is required: items inside are `pub(crate)`, and `pub(crate)`
// in a private module trips `redundant_pub_crate` while `pub` trips `unreachable_pub`.
pub mod cli;
pub mod commands;
pub mod fmt;
pub mod output;
pub mod progress;

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::Verbosity;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    init_tracing(args.verbosity)?;
    commands::dispatch(args).await
}

fn init_tracing(verbosity: Verbosity) -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(verbosity.tracing_level_filter().into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))
        .context("installing tracing subscriber")
}
