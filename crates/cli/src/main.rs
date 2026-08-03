//! imsg CLI — entry point for the iMessage/MAP client over Bluetooth MAP/PBAP.

use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::Verbosity;

use imsg::cli::Cli;
use imsg::commands;

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
