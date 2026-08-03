//! Generates static shell completion files for every `clap_complete`-supported shell into
//! `completions/`.
//!
//! Run via `just gen-completions`. Checked into the repo (like `tools/setup-*`) and bundled
//! into releases via `dist-workspace.toml`'s `extra-artifacts`.

use std::path::Path;

use clap::{CommandFactory as _, ValueEnum as _};
use clap_complete::{generate_to, Shell};
use imsg::cli::Cli;

fn main() -> anyhow::Result<()> {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../completions");
    std::fs::create_dir_all(&out_dir)?;

    let mut cmd = Cli::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "imsg", &out_dir)?;
    }
    Ok(())
}
