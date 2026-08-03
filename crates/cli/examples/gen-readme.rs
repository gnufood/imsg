//! Regenerates the `## Commands` section of `README.md` from the `imsg` CLI's clap definition.
//!
//! Run via `just docs`. Keeps the README's command reference in sync with `cli.rs` without
//! hand-maintaining a duplicate table.

use std::path::Path;

use imsg::cli::Cli;

const START: &str = "<!-- COMMANDS:START -->";
const END: &str = "<!-- COMMANDS:END -->";

fn main() -> anyhow::Result<()> {
    let opts = clap_markdown::MarkdownOptions::new().show_footer(false);
    let generated = clap_markdown::help_markdown_custom::<Cli>(&opts);

    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
    let readme = std::fs::read_to_string(&readme_path)?;
    let (before, rest) = readme
        .split_once(START)
        .ok_or_else(|| anyhow::anyhow!("missing {START} marker in README.md"))?;
    let (_, after) =
        rest.split_once(END).ok_or_else(|| anyhow::anyhow!("missing {END} marker in README.md"))?;

    let updated = format!("{before}{START}\n\n{generated}\n{END}{after}");
    std::fs::write(&readme_path, updated)?;
    Ok(())
}
