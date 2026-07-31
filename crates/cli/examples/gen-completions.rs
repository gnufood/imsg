//! Generates static shell completion files (bash/zsh/fish/elvish) into `completions/`.
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
    // PowerShell completions aren't shipped — N/A per TODO.md until there's a PowerShell user to
    // validate against.
    for &shell in Shell::value_variants().iter().filter(|&&shell| shell != Shell::PowerShell) {
        generate_to(shell, &mut cmd, "imsg", &out_dir)?;
    }
    Ok(())
}
